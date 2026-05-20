use crate::model::folder::ViewedFolder;
use crate::model::front::ViewedFrontEntry;
use crate::model::member::ViewedMember;
use serde::{Deserialize, Serialize};
use crate::model::{validate_color_range, validate_string_length};

pub type UserId = i32;

#[derive(Debug, Deserialize)]
pub struct UserFilter {
    #[serde(rename = "userId")]
    pub user_id: Option<UserId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(skip_deserializing)]
    pub id: UserId,
    pub name: String,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub color: u32,
    pub system: bool,
}

impl UserInfo {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("name", &self.name, Some(3), Some(50), false)?;
        if let Some(avatar) = &self.avatar {
            validate_string_length("avatar", avatar, Some(1), Some(255), true)?;
        }
        if let Some(description) = &self.description {
            validate_string_length("description", description, Some(1), Some(65535), true)?;
        }
        validate_color_range("color", self.color)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ExtendedUserInfo {
    pub user: UserInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folders: Option<Vec<ViewedFolder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<ViewedMember>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub front: Option<Vec<ViewedFrontEntry>>,
}