use crate::error::WebError;
use actix_web::{get, HttpRequest};
use serde_json::json;
use crate::security::{random_string, sha512};
use crate::web::{ok, WebResult};

pub mod password;
pub mod stats;
pub mod user;

const ADMIN_SECRET_TOKEN_SHA: &'static str = env!("ADMIN_SECRET_TOKEN_SHA");
const ADMIN_SECRET_TOKEN_LENGTH: usize = 16384;

pub(in crate::web::admin) fn verify_admin_token(req: &HttpRequest) -> Result<(), WebError> {
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth) = auth.to_str() {
            if let Some((token_type, token)) = auth.split_once(' ') {
                if token_type == "Bearer" {
                    let hash = sha512(token);
                    if token.len() == ADMIN_SECRET_TOKEN_LENGTH && hash == ADMIN_SECRET_TOKEN_SHA {
                        return Ok(());
                    }
                }
            }
        }
    }
    Err(WebError::InvalidToken)
}

#[get("/regen")]
pub async fn regenerate_admin_token() -> WebResult {
    let token = random_string(ADMIN_SECRET_TOKEN_LENGTH);
    let hash = sha512(&token);
    ok(json!({
        "token": token,
        "hash": hash,
    }))
}