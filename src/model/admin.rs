use serde::{Deserialize, Serialize};
use crate::model::user::UserId;

#[derive(Deserialize)]
pub struct AdminMakePasswordResetTokenRequest {
    pub secret: String,
    pub user: UserId,
}

#[derive(Serialize)]
pub struct AdminTokenResponse {
    pub token: String,
}