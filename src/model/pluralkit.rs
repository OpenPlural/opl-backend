use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PkConfig {
    pub token: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}