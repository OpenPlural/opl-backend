use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::validate_string_length;
use crate::model::member::MemberId;
use crate::model::user::UserId;

pub type PhotoAlbumId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoAlbum {
    #[serde(skip_deserializing)]
    pub id: PhotoAlbumId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(skip_deserializing, rename = "memberId")]
    pub member_id: MemberId,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub sort: u16,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "photoUrls")]
    pub photo_urls: Option<Vec<String>>,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
}

impl PhotoAlbum {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("PhotoAlbum", "name", &self.name, Some(1), Some(255), false)?;
        if let Some(description) = &self.description {
            validate_string_length("PhotoAlbum", "description", description, Some(1), Some(65535), true)?;
        }
        if let Some(photo_urls) = &self.photo_urls {
            if photo_urls.is_empty() {
                return Err("PhotoAlbum.photoUrls must not be empty. If you want to leave this field empty, please set it to NULL instead.".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ViewedPhotoAlbum {
    pub id: PhotoAlbumId,
    pub sort: u16,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "photoUrls")]
    pub photo_urls: Option<Vec<String>>
}