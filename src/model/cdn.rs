use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: String,
    pub access: String,
}

#[derive(Debug, Deserialize)]
pub struct AvatarAccessQuery {
    pub access: String,
    pub week: i64,
}