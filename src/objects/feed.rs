use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Feed {
    pub val: FeedVal,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FeedVal {
    pub id: Uuid,
    pub url: String,
    pub update_ts: DateTime<FixedOffset>,
}
