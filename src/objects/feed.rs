use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: Uuid,
    pub url: String,
    pub update_ts: DateTime<FixedOffset>,
}
