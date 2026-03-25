use crate::model::folder::FolderId;
use crate::model::user::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type MemberId = i32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    #[serde(skip_deserializing)]
    pub id: MemberId,
    #[serde(rename = "userId", skip_deserializing)]
    pub user_id: UserId,
    pub name: String,
    pub pronouns: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub color: u32,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub custom: bool,
    #[serde(rename = "createdAt", skip_deserializing)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_deserializing)]
    pub folders: Vec<FolderId>,
}