use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::member::MemberId;
use crate::model::user::UserId;

pub type FrontEntryId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontEntry {
    pub id: FrontEntryId,
    #[serde(skip)]
    pub user: UserId,
    pub member: MemberId,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "endedAt")]
    pub ended_at: Option<String>,
    pub comment: Option<String>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: Option<DateTime<Utc>>,
}