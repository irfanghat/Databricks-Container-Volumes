use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadSession {
    pub id: Uuid,
    pub repository: String,
    pub offset: u64,
}
