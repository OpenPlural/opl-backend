use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::user::UserId;
use crate::model::validate_string_length;

pub type ApiKeyId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKey {
    #[serde(skip_deserializing)]
    pub id: ApiKeyId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub name: String,
    pub write: bool,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none", rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

impl ApiKey {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("Member", "name", &self.name, Some(1), Some(255), false)?;
        Ok(())
    }
}