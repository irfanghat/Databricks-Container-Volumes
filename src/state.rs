use std::sync::Arc;

use crate::storage::VolumeStorage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<VolumeStorage>,
}
