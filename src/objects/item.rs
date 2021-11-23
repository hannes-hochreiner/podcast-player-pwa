use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub val: ItemVal,
    pub meta: ItemMeta,
    pub keys: ItemKeys,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemKeys {
    pub id: Uuid,
    pub year_month: String,
}

impl From<ItemVal> for ItemKeys {
    fn from(val: ItemVal) -> Self {
        Self {
            id: val.id,
            year_month: val.date.to_rfc3339()[0..7].to_string(),
        }
    }
}
