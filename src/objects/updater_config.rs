use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    pub last_fetch_feeds: Option<DateTime<FixedOffset>>,
    pub last_fetch_channels: Option<DateTime<FixedOffset>>,
    pub last_fetch_items: Option<DateTime<FixedOffset>>,
}
