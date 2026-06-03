use chrono::{DateTime, Utc};
use serde::Serialize;

pub type TokenId = i32;

#[derive(Debug, Serialize)]
pub struct SessionToken {
    pub id: TokenId,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: DateTime<Utc>,
}