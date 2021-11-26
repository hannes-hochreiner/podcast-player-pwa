use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Channel {
    pub val: ChannelVal,
    pub meta: ChannelMeta,
    pub keys: ChannelKeys,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelVal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub image: String,
    pub feed_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelMeta {
    pub id: Uuid,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelKeys {
    pub id: Uuid,
    pub year_month_keys: HashSet<String>,
}
