use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::{
    Client, RequestBuilder, Response,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use uuid::Uuid;

use crate::session::UploadSession;

pub struct VolumeStorage {
    client: Client,
    workspace_url: String,
    token: String,
    volume_root: String,
}

impl VolumeStorage {
    pub fn new(
        workspace_url: impl Into<String>,
        token: impl Into<String>,
        volume_root: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            workspace_url: workspace_url.into().trim_end_matches('/').to_string(),
            token: token.into(),
            volume_root: volume_root.into().trim_end_matches('/').to_string(),
        }
    }

    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        req.header(AUTHORIZATION, format!("Bearer {}", self.token))
    }

    fn file_url(&self, path: &str) -> String {
        format!("{}/api/2.0/fs/files{}", self.workspace_url, path)
    }

    fn directory_url(&self, path: &str) -> String {
        format!("{}/api/2.0/fs/directories{}", self.workspace_url, path)
    }

    fn volume_path(&self, relative: &str) -> String {
        format!("{}/{}", self.volume_root, relative)
    }

    //
    // ------------------------------------------------------------------------
    // Generic filesystem operations
    // ------------------------------------------------------------------------
    //

    pub async fn create_directory(&self, relative: &str) -> Result<()> {
        let path = self.volume_path(relative);

        self.auth(self.client.put(self.directory_url(&path)))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn upload_file(&self, relative: &str, bytes: Bytes) -> Result<()> {
        let path = self.volume_path(relative);

        self.auth(
            self.client
                .put(self.file_url(&path))
                .header(CONTENT_TYPE, "application/octet-stream")
                .body(bytes),
        )
        .send()
        .await?
        .error_for_status()
        .context("upload failed")?;

        Ok(())
    }

    pub async fn download_file(&self, relative: &str) -> Result<Response> {
        let path = self.volume_path(relative);

        Ok(self
            .auth(self.client.get(self.file_url(&path)))
            .send()
            .await?
            .error_for_status()?)
    }

    pub async fn file_exists(&self, relative: &str) -> Result<bool> {
        let path = self.volume_path(relative);

        let response = self
            .auth(self.client.head(self.file_url(&path)))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn delete_file(&self, relative: &str) -> Result<()> {
        let path = self.volume_path(relative);

        self.auth(self.client.delete(self.file_url(&path)))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    //
    // ------------------------------------------------------------------------
    // OCI helpers
    // ------------------------------------------------------------------------
    //

    pub async fn get_session(&self, id: &Uuid) -> Result<UploadSession> {
        let path = format!("uploads/{id}/session.json");
        let resp = self.download_file(&path).await?;
        let bytes = resp.bytes().await?;
        let session = serde_json::from_slice(&bytes)?;
        Ok(session)
    }

    pub async fn update_session(&self, session: &UploadSession) -> Result<()> {
        let json = serde_json::to_vec_pretty(session)?;
        self.upload_file(
            &format!("uploads/{}/session.json", session.id),
            Bytes::from(json),
        )
        .await?;
        Ok(())
    }

    pub async fn create_upload(&self, repository: &str) -> Result<Uuid> {
        let id = Uuid::new_v4();

        self.create_directory(&format!("uploads/{id}")).await?;

        let session = UploadSession {
            id,
            repository: repository.to_string(),
            offset: 0,
        };

        let json = serde_json::to_vec_pretty(&session)?;

        self.upload_file(&format!("uploads/{id}/session.json"), Bytes::from(json))
            .await?;

        Ok(id)
    }

    pub async fn commit_upload(&self, id: &Uuid, digest: &str) -> Result<()> {
        let upload_path = self.upload_path(id);
        let blob_path = self.blob_path(digest);

        let resp = self.download_file(&upload_path).await?;
        let bytes = resp.bytes().await?;

        // Ensure parent dirs for blob path exist
        let hash = digest.trim_start_matches("sha256:");
        self.create_directory(&format!("blobs/sha256/{}", &hash[..2]))
            .await?;

        self.upload_file(&blob_path, bytes).await?;

        // Cleanup
        let _ = self.delete_file(&upload_path).await;
        let _ = self
            .delete_file(&format!("uploads/{id}/session.json"))
            .await;

        Ok(())
    }

    pub fn upload_path(&self, id: &Uuid) -> String {
        format!("uploads/{id}/data")
    }

    pub fn blob_path(&self, digest: &str) -> String {
        let hash = digest.trim_start_matches("sha256:");

        format!("blobs/sha256/{}/{}", &hash[..2], hash,)
    }

    pub fn manifest_path(&self, repo: &str, reference: &str) -> String {
        format!("repositories/{}/manifests/{}.json", repo, reference,)
    }

    pub async fn initialize(&self) -> Result<()> {
        self.create_directory("uploads").await?;
        self.create_directory("blobs").await?;
        self.create_directory("blobs/sha256").await?;
        self.create_directory("repositories").await?;

        Ok(())
    }
}
