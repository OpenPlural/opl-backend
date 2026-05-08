use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::user::UserId;

pub type FolderId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "parentId")]
    pub parent_id: Option<FolderId>,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    pub color: u32,
    #[serde(rename = "createdAt", skip_deserializing)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: Option<DateTime<Utc>>,
}