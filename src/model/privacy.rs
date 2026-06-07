use crate::model::folder::FolderId;
use crate::model::member::MemberId;
use crate::model::user::UserId;
use crate::model::{validate_color_range, validate_string_length};
use serde::{Deserialize, Serialize};

pub type PrivacyBucketId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivacyBucket {
    #[serde(skip_deserializing)]
    pub id: PrivacyBucketId,
    #[serde(skip)]
    pub user_id: UserId,
    pub sort: u16,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    pub color: u32,
    #[serde(skip_deserializing)]
    pub folders: Vec<FolderId>,
    #[serde(skip_deserializing)]
    pub members: Vec<MemberId>,
    #[serde(skip_deserializing)]
    pub friends: Vec<UserId>,
}

impl PrivacyBucket {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("PrivacyBucket", "name", &self.name, Some(1), Some(255), false)?;
        if let Some(description) = &self.description {
            validate_string_length("PrivacyBucket", "description", description, Some(1), Some(65535), true)?;
        }
        if let Some(emoji) = &self.emoji {
            validate_string_length("PrivacyBucket", "emoji", emoji, Some(1), Some(4), true)?;
        }
        validate_color_range("PrivacyBucket", "color", self.color)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct SimplePrivacyBucket {
    pub id: PrivacyBucketId,
    pub sort: u16,
    pub name: String,
    pub emoji: Option<String>,
    pub color: u32,
}