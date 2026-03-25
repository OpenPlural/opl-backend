use serde::{Deserialize, Serialize};
use crate::model::folder::Folder;
use crate::model::front::FrontEntry;
use crate::model::member::Member;
use crate::model::user::UserId;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
    pub system: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub device: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionResponse>,
    pub id: UserId,
    pub name: String,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub color: u32,
    pub system: bool,
    #[serde(rename = "friendCode", skip_serializing_if = "Option::is_none")]
    pub friend_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folders: Option<Vec<Folder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<Member>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub front: Option<Vec<FrontEntry>>,
}