use chrono::{offset::FixedOffset, DateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FetcherConfig {
    pub authorization_task: Option<AuthorizationTask>,
    pub authorization: Option<Authorization>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthorizationTask {
    pub verifier: String,
    pub challenge: String,
    pub state: Uuid,
    pub redirect: String,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            authorization_task: None,
            authorization: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Authorization {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<FixedOffset>,
}
