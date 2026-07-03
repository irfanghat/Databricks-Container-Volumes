use axum::{
    http::{HeaderMap, StatusCode, header::HeaderName},
    response::IntoResponse,
};

pub async fn v2() -> impl IntoResponse {
    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_static("docker-distribution-api-version"),
        "registry/2.0".parse().unwrap(),
    );

    (StatusCode::OK, headers)
}
