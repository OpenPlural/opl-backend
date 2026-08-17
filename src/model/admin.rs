use chrono::{DateTime, Utc};
use crate::model::folder::Folder;
use crate::model::front::FrontEntry;
use crate::model::member::Member;
use crate::model::user::{UserId, UserInfo};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminMakePasswordResetTokenRequest {
    pub user: UserId,
}

#[derive(Deserialize)]
pub struct AdminChangePasswordRequest {
    pub user: UserId,
    pub password: String,
}

#[derive(Serialize)]
pub struct AdminTokenResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct AdminStatisticsResponse {
    pub users: i64,
    pub folders: i64,
    pub members: i64,
    pub fields: i64,
    #[serde(rename = "fieldEntries")]
    pub field_entries: i64,
    pub fronts: i64,
    #[serde(rename = "frontMembers")]
    pub front_members: i64,
    pub friends: i64,
    pub deletions: i64,
    #[serde(rename = "privacyBuckets")]
    pub privacy_buckets: i64,
    pub sessions: i64,
    #[serde(rename = "sessionUsers")]
    pub session_users: i64,
    #[serde(rename = "apiKeys")]
    pub api_keys: i64,
    #[serde(rename = "databaseTime")]
    pub database_time: DateTime<Utc>,
    #[serde(rename = "serverTime")]
    pub server_time: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminUserListEntry {
    pub id: UserId,
    pub name: String,
    pub email: Option<String>,
    pub color: u32,
    pub system: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminUserInfo {
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "friendCode")]
    pub friend_code: String,
    pub disabled: bool,
    #[serde(rename = "canResetPassword")]
    pub can_reset_password: bool,
    pub user: UserInfo,
    pub folders: Vec<Folder>,
    pub members: Vec<Member>,
    pub front: Vec<FrontEntry>,
}