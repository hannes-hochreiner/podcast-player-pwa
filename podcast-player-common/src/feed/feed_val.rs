#[cfg(feature = "tokio-postgres")]
use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tokio-postgres")]
use std::convert::TryFrom;
#[cfg(feature = "tokio-postgres")]
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedVal {
    pub id: Uuid,
    pub url: String,
    pub update_ts: DateTime<FixedOffset>,
}

#[cfg(feature = "tokio-postgres")]
impl TryFrom<&Row> for FeedVal {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            url: row.try_get("url")?,
            update_ts: row.try_get("update_ts")?,
        })
    }
}
