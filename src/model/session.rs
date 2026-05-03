use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::middleware::TokenId;

#[derive(Serialize)]
pub struct SessionToken {
    pub id: TokenId,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}