#[cfg(feature = "tokio-postgres")]
use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tokio-postgres")]
use std::convert::TryFrom;
#[cfg(feature = "tokio-postgres")]
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelVal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub feed_id: Uuid,
    pub update_ts: DateTime<FixedOffset>,
}

impl ChannelVal {
    pub fn needs_update(&self, description: &String, image: &Option<String>) -> bool {
        if &self.description == description && &self.image == image {
            false
        } else {
            true
        }
    }
}

#[cfg(feature = "tokio-postgres")]
impl TryFrom<&Row> for ChannelVal {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            description: row.try_get("description")?,
            title: row.try_get("title")?,
            image: match row.try_get("image") {
                Ok(i) => Some(i),
                Err(_) => None,
            },
            feed_id: row.try_get("feed_id")?,
            update_ts: row.try_get("update_ts")?,
        })
    }
}
