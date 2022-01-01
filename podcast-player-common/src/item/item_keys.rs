use super::{item_meta::DownloadStatus, ItemMeta, ItemVal};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemKeys {
    pub id: Uuid,
    pub year_month: String,
    pub download_required: String,
    pub download_ok: String,
}

impl ItemKeys {
    pub fn new_from_val_meta(val: &ItemVal, meta: &ItemMeta) -> Self {
        let download_required = match (&meta.download, &meta.download_status) {
            (true, DownloadStatus::Pending) => String::from("true"),
            _ => String::from("false"),
        };
        let download_ok = match &meta.download_status {
            &DownloadStatus::Ok(_) => String::from("true"),
            _ => String::from("false"),
        };

        Self {
            id: val.id,
            year_month: val.date.to_rfc3339()[0..7].to_string(),
            download_required,
            download_ok,
        }
    }
}
