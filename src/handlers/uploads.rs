use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub async fn start_upload(
    Path(repo): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("POST upload repo={}", repo);

    let id = state.storage.create_upload(&repo).await.unwrap();

    let mut headers = HeaderMap::new();

    headers.insert(
        "Docker-Distribution-API-Version",
        HeaderValue::from_static("registry/2.0"),
    );

    headers.insert(
        "Location",
        HeaderValue::from_str(&format!("/v2/{repo}/blobs/uploads/{id}")).unwrap(),
    );

    headers.insert(
        "Docker-Upload-UUID",
        HeaderValue::from_str(&id.to_string()).unwrap(),
    );

    headers.insert("Range", HeaderValue::from_static("0-0"));

    (StatusCode::ACCEPTED, headers)
}

pub async fn patch_upload(
    Path((repo, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("PATCH upload repo={} id={}", repo, id);

    let mut session = state.storage.get_session(&id).await.unwrap();
    let upload_path = state.storage.upload_path(&id);

    // Databricks Volume FS doesn't support append, so we read-modify-write.
    // For a production system, buffering or multipart APIs should be used.
    let mut data = if session.offset > 0 {
        state
            .storage
            .download_file(&upload_path)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .to_vec()
    } else {
        Vec::new()
    };

    data.extend_from_slice(&body);
    let new_offset = data.len() as u64;

    state
        .storage
        .upload_file(&upload_path, Bytes::from(data))
        .await
        .unwrap();

    session.offset = new_offset;
    state.storage.update_session(&session).await.unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Location",
        HeaderValue::from_str(&format!("/v2/{repo}/blobs/uploads/{id}")).unwrap(),
    );
    headers.insert(
        "Range",
        HeaderValue::from_str(&format!("0-{}", new_offset.saturating_sub(1))).unwrap(),
    );
    headers.insert(
        "Docker-Upload-UUID",
        HeaderValue::from_str(&id.to_string()).unwrap(),
    );

    (StatusCode::ACCEPTED, headers)
}

#[derive(Deserialize)]
pub struct PutQuery {
    digest: String,
}

pub async fn put_upload(
    Path((repo, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Query(params): Query<PutQuery>,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!(
        "PUT upload repo={} id={} digest={}",
        repo,
        id,
        params.digest
    );

    // Monolithic PUT or final chunk
    if !body.is_empty() {
        patch_upload(Path((repo.clone(), id)), State(state.clone()), body).await;
    }

    state
        .storage
        .commit_upload(&id, &params.digest)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Location",
        HeaderValue::from_str(&format!("/v2/{repo}/blobs/{}", params.digest)).unwrap(),
    );
    headers.insert(
        "Docker-Content-Digest",
        HeaderValue::from_str(&params.digest).unwrap(),
    );

    (StatusCode::CREATED, headers)
}
