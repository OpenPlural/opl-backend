use crate::model::session::TokenId;
use crate::model::user::{UserId, UserInfo};
use crate::model::validate_string_length;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const MIN_NAME_LENGTH: usize = 3;
const MAX_NAME_LENGTH: usize = 50;
const MIN_PASSWORD_LENGTH: usize = 10;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
    pub system: bool,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("name", &self.name, Some(MIN_NAME_LENGTH), Some(MAX_NAME_LENGTH), false)?;
        validate_string_length("password", &self.password, Some(MIN_PASSWORD_LENGTH), None, false)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub device: String,
    pub name: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("device", &self.device, None, Some(255), false)?;
        validate_string_length("name", &self.name, Some(MIN_NAME_LENGTH), Some(MAX_NAME_LENGTH), false)?;
        validate_string_length("password", &self.password, Some(MIN_PASSWORD_LENGTH), None, false)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub id: UserId,
    pub password: String,
}

impl DeleteRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("password", &self.password, Some(MIN_PASSWORD_LENGTH), None, false)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub id: UserId,
    #[serde(rename = "oldPassword")]
    pub old_password: String,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

impl ChangePasswordRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("oldPassword", &self.old_password, Some(MIN_PASSWORD_LENGTH), None, false)?;
        validate_string_length("newPassword", &self.new_password, Some(MIN_PASSWORD_LENGTH), None, false)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: TokenId,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub session: SessionResponse,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "friendCode")]
    pub friend_code: String,
    pub user: UserInfo,
}