use serde::{Deserialize, Serialize};
use crate::model::member::MemberId;

pub type FrontEntryId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontEntry {
    #[serde(skip_deserializing)]
    pub id: FrontEntryId,
    pub member: MemberId,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "endedAt")]
    pub ended_at: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FrontCommentRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FrontStartTimeRequest {
    #[serde(rename = "startedAt")]
    pub started_at: String,
}

#[derive(Debug, Deserialize)]
pub struct FrontEndTimeRequest {
    #[serde(rename = "endedAt")]
    pub ended_at: Option<String>,
}