use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};

use crate::state::AppState;

pub async fn start_upload(
    Path(repo): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let id = state.storage.create_upload().await.unwrap();

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
