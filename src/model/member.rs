use crate::model::folder::{FolderId, ViewedFolder};
use crate::model::user::UserId;
use crate::model::{validate_color_range, validate_string_length};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type MemberId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    #[serde(skip_deserializing)]
    pub id: MemberId,
    #[serde(skip)]
    pub user_id: UserId,
    pub name: String,
    pub pronouns: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub custom: bool,
    #[serde(rename = "createdAt", skip_deserializing)]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_deserializing)]
    pub folders: Vec<FolderId>,
}

#[derive(Debug, Serialize)]
pub struct ViewedMember {
    pub id: MemberId,
    pub name: String,
    pub pronouns: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub color: u32,
    pub archived: bool,
    pub folders: Vec<FolderId>,
}

impl From<Member> for ViewedMember {
    fn from(member: Member) -> Self {
        Self {
            id: member.id,
            name: member.name,
            pronouns: member.pronouns,
            avatar: member.avatar,
            description: member.description,
            color: member.color,
            archived: member.archived,
            folders: member.folders,
        }
    }
}

impl Member {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("Member", "name", &self.name, Some(1), Some(255), false)?;
        if let Some(pronouns) = &self.pronouns {
            validate_string_length("Member", "pronouns", pronouns, Some(1), Some(255), true)?;
        }
        if let Some(avatar) = &self.avatar {
            validate_string_length("Member", "avatar", avatar, Some(1), Some(255), true)?;
        }
        if let Some(description) = &self.description {
            validate_string_length("Member", "description", description, Some(1), Some(65535), true)?;
        }
        validate_color_range("Member", "color", self.color)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ExtendedViewedMember {
    pub member: ViewedMember,
    pub folders: Vec<ViewedFolder>,
}

#[derive(Debug, Deserialize)]
pub struct MemberQuery {
    #[serde(rename = "userId")]
    pub user_id: Option<UserId>,
    #[serde(default)]
    pub extended: bool,
}