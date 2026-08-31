use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::member::MemberId;
use crate::model::user::UserId;
use crate::model::validate_string_length;

pub type FrontEntryId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontEntry {
    #[serde(skip_deserializing)]
    pub id: FrontEntryId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "member", deserialize_with = "crate::numberstring::deserialize")]
    pub member_id: MemberId,
    #[serde(rename = "startedAt")]
    pub started_at: DateTime<Utc>,
    #[serde(rename = "endedAt")]
    pub ended_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
}

impl FrontEntry {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(comment) = &self.comment {
            validate_string_length("FrontEntry", "comment", comment, Some(1), Some(255), true)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ViewedFrontEntry {
    pub id: FrontEntryId,
    #[serde(rename = "member")]
    pub member_id: MemberId,
    pub comment: Option<String>,
}

impl From<FrontEntry> for ViewedFrontEntry {
    fn from(entry: FrontEntry) -> Self {
        Self {
            id: entry.id,
            member_id: entry.member_id,
            comment: entry.comment,
        }
    }
}