use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnclosureMeta {
    pub id: Uuid,
    pub download_result: anyhow::Result<u32>,
}
