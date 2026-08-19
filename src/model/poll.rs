use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::user::UserId;
use crate::model::{validate_number_range, validate_string_length};
use crate::model::member::MemberId;

pub type PollId = i64;
pub type PollAnswerId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    #[serde(skip_deserializing)]
    pub id: PollId,
    #[serde(skip)]
    pub user_id: UserId,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "allowAbstain")]
    pub allow_abstain: bool,
    #[serde(rename = "allowVeto")]
    pub allow_veto: bool,
    #[serde(rename = "openUntil")]
    pub open_until: DateTime<Utc>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "customOptions", default)]
    pub custom_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollAnswer {
    #[serde(skip_deserializing)]
    pub id: PollAnswerId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "pollId")]
    pub poll_id: PollId,
    #[serde(rename = "memberId")]
    pub member_id: MemberId,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub answer: u8,
    pub comment: Option<String>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
}

impl Poll {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("Poll", "name", &self.name, Some(1), Some(255), false)?;
        if let Some(description) = &self.description {
            validate_string_length("Poll", "description", description, Some(1), Some(65535), true)?;
        }
        if let Some(custom_options) = &self.custom_options {
            validate_string_length("Poll", "custom_options", &custom_options.join(""), Some(1), Some(65000), true)?;
            validate_number_range("Poll", "Length(custom_options)", custom_options.len() as isize, 2, 16)?;
        }
        Ok(())
    }
}

impl PollAnswer {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(comment) = &self.comment {
            validate_string_length("PollAnswer", "comment", comment, Some(1), Some(255), true)?;
        }
        Ok(())
    }
}