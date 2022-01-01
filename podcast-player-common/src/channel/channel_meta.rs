use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelMeta {
    pub id: Uuid,
    pub active: bool,
}
