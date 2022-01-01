use chrono::{offset::FixedOffset, DateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FetcherConfig {
    pub authorization_task: Option<AuthorizationTask>,
    pub authorization: Option<Authorization>,
    pub config: AuthorizationConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthorizationTask {
    pub verifier: String,
    pub challenge: String,
    pub state: Uuid,
    pub redirect: String,
}

impl FetcherConfig {
    pub fn new_with_config(config: AuthorizationConfig) -> Self {
        Self {
            authorization_task: None,
            authorization: None,
            config,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Authorization {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthorizationConfig {
    pub audience: String,
    pub client_id: String,
    pub domain: String,
}
