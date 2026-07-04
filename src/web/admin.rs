use actix_web::post;
use actix_web::web::{Data, Json};
use crate::AppState;
use crate::database::to_web_error;
use crate::model::admin::{AdminMakePasswordResetTokenRequest, AdminTokenResponse};
use crate::security::{random_string, sha256, verify_admin_token};
use crate::web::{ok, WebResult};

#[post("/make-password-reset-token")]
pub async fn make_password_reset_token(req: Json<AdminMakePasswordResetTokenRequest>, data: Data<AppState>) -> WebResult {
    verify_admin_token(&req.secret)?;

    let token = random_string(128);
    let token_hash = sha256(&token);
    crate::database::admin::update_password_reset_token(&data.pool, req.user, &token_hash).await.map_err(to_web_error)?;

    ok(AdminTokenResponse {
        token,
    })
}