use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelKeys {
    pub id: Uuid,
    pub year_month_keys: HashSet<String>,
}
