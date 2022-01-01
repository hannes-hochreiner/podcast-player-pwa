use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemMeta {
    pub id: Uuid,
    pub new: bool,
    pub download: bool,
    pub download_status: DownloadStatus,
    pub current_time: Option<f64>,
    pub play_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DownloadStatus {
    NotRequested,
    Pending,
    InProgress,
    Ok(u32),
    Error,
}
