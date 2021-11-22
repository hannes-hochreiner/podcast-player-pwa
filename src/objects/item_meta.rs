use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemMeta {
    pub id: Uuid,
    pub new: bool,
    pub download: bool,
}
