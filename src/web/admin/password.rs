use actix_web::{post, HttpRequest};
use actix_web::web::{Data, Json};
use crate::AppState;
use crate::database::to_web_error;
use crate::model::admin::{AdminChangePasswordRequest, AdminMakePasswordResetTokenRequest, AdminTokenResponse};
use crate::security::{random_string, sha256};
use crate::web::{ok, ok_none, WebResult};
use crate::web::admin::verify_admin_token;

#[post("/make-reset-token")]
pub async fn make_password_reset_token(req: HttpRequest, data: Data<AppState>, body: Json<AdminMakePasswordResetTokenRequest>) -> WebResult {
    verify_admin_token(&req)?;

    let token = random_string(128);
    let token_hash = sha256(&token);
    crate::database::admin::update_password_reset_token(&data.pool, body.user, &token_hash).await.map_err(to_web_error)?;

    ok(AdminTokenResponse {
        token,
    })
}

#[post("/revoke-reset-token")]
pub async fn revoke_password_reset_token(req: HttpRequest, data: Data<AppState>, body: Json<AdminMakePasswordResetTokenRequest>) -> WebResult {
    verify_admin_token(&req)?;

    crate::database::admin::clear_password_reset_token(&data.pool, body.user).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/change-password")]
pub async fn force_change_password(req: HttpRequest, data: Data<AppState>, body: Json<AdminChangePasswordRequest>) -> WebResult {
    verify_admin_token(&req)?;

    crate::database::user::change_password(&data.pool, body.user, &body.password).await.map_err(to_web_error)?;
    ok_none()
}