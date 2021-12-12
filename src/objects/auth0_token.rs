use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth0Token {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
}
