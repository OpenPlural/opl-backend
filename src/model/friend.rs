use crate::model::user::UserInfo;
use crate::model::validate_number_range;
use serde::{Deserialize, Serialize};

pub const PERMISSION_LEVEL_MEMBERS: i8 = 1;
pub const PERMISSION_LEVEL_FRONT: i8 = 2;
pub const PERMISSION_LEVEL_NOTIFICATIONS: i8 = 3;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FriendSettings {
    #[serde(rename = "permissionLevel")]
    pub permission_level: i8,
    #[serde(rename = "notifyMe")]
    pub notify_me: bool,
}

impl FriendSettings {
    pub fn validate(&self) -> Result<(), String> {
        validate_number_range("FriendSettings", "permissionLevel", self.permission_level as isize, 0, 3)?;
        Ok(())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct FriendRequest {
    pub code: String,
    pub name: String,
    pub system: bool,
}

#[derive(Debug, Serialize)]
pub struct Friend {
    pub user: UserInfo,
    #[serde(rename = "frontText")]
    pub front_text: Option<String>,
}