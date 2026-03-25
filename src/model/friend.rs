use serde::{Deserialize, Serialize};

pub const PERMISSION_LEVEL_NONE: i8 = 0;
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

#[derive(Debug, Default, Serialize)]
pub struct FriendRequest {
    pub code: String,
    pub name: String,
}

impl FriendSettings {
    pub fn check_permission(&self, level: i8) -> bool {
        self.permission_level >= level
    }
}