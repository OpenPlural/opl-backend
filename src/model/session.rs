use chrono::{DateTime, Utc};
use serde::Serialize;

pub type SessionId = i64;

#[derive(Debug, Serialize)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: DateTime<Utc>,
}