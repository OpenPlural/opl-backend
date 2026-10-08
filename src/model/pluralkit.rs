use serde::{Deserialize, Serialize};
use crate::model::validate_string_length;

#[derive(Debug, Deserialize)]
pub struct PkSyncQuery {
    #[serde(default)]
    pub direction: PkSyncDirection
}

#[derive(Debug, Default, Deserialize)]
pub enum PkSyncDirection {
    #[default]
    Push,
    Pull,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PkConfig {
    pub token: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

impl PkConfig {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(token) = &self.token {
            validate_string_length("PkConfig", "token", token, Some(1), Some(64), true)?;
        }
        if let Some(display_name) = &self.display_name {
            validate_string_length("PkConfig", "displayName", display_name, Some(1), Some(255), true)?;
        }
        Ok(())
    }
}