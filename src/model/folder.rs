use crate::model::user::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::{validate_color_range, validate_string_length};

pub type FolderId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    #[serde(skip_deserializing)]
    pub id: FolderId,
    #[serde(skip)]
    pub user_id: UserId,
    pub sort: u16,
    #[serde(rename = "parentId", deserialize_with = "crate::numberstring::deserialize_opt")]
    pub parent_id: Option<FolderId>,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
    #[serde(rename = "createdAt", skip_deserializing)]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ViewedFolder {
    pub id: FolderId,
    pub sort: u16,
    #[serde(rename = "parentId")]
    pub parent_id: Option<FolderId>,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    pub color: u32,
}

impl From<Folder> for ViewedFolder {
    fn from(folder: Folder) -> Self {
        Self {
            id: folder.id,
            sort: folder.sort,
            parent_id: folder.parent_id,
            name: folder.name,
            description: folder.description,
            emoji: folder.emoji,
            color: folder.color,
        }
    }
}

impl Folder {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("Folder", "name", &self.name, Some(1), Some(255), false)?;
        if let Some(description) = &self.description {
            validate_string_length("Folder", "description", description, Some(1), Some(65535), true)?;
        }
        if let Some(emoji) = &self.emoji {
            validate_string_length("Folder", "emoji", emoji, Some(1), Some(30), true)?;
        }
        validate_color_range("Folder", "color", self.color)?;
        Ok(())
    }
}