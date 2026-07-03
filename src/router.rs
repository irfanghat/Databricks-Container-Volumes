use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    handlers::{base, uploads},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v2/", get(base::v2))
        .route("/v2/{repo}/blobs/uploads/", post(uploads::start_upload))
}
