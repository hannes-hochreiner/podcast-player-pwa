use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub val: ItemVal,
    pub meta: ItemMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemVal {
    pub id: Uuid,
    pub title: String,
    pub date: DateTime<FixedOffset>,
    pub enclosure_type: String,
    pub enclosure_url: String,
    pub channel_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemMeta {
    pub id: Uuid,
    pub new: bool,
    pub download: bool,
}
