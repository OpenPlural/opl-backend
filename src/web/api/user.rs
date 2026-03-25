use crate::database::session::extend_session;
use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::{get_token, RequestToken};
use crate::model::friend::PERMISSION_LEVEL_FRONT;
use crate::model::user::{FullDataSelectionFilter, UserFilter};
use crate::web::{ok, WebResult};
use crate::AppState;
use actix_web::web::{Data, Query};
use actix_web::{get, HttpRequest};

#[get("/self")]
pub async fn get_self_user(req: HttpRequest, data: Data<AppState>, query: Query<FullDataSelectionFilter>) -> WebResult {
    // This endpoint is protected by the bearer middleware, so if we get here, the token is valid

    let token: RequestToken = get_token(&req).unwrap();
    if let Some(mut res) = crate::database::user::get_user_by_id(&data.pool, token.user_id, true).await.map_err(to_web_error)? {
        extend_session(&data.pool, token.token_id).await.map_err(to_web_error)?;

        if query.full.unwrap_or(false) {
            res.folders = Some(crate::database::folder::get_folders(&data.pool, token.user_id).await.map_err(to_web_error)?);
            res.members = Some(crate::database::member::get_members(&data.pool, token.user_id).await.map_err(to_web_error)?);
            res.front = Some(crate::database::front::get_current_front_entries(&data.pool, token.user_id).await.map_err(to_web_error)?);
        }

        ok(res)
    } else {
        // This should never happen, because of database foreign key enforcements, but we handle it just in case
        Err(WebError::InvalidToken)
    }
}

#[get("/front")]
pub async fn get_front(req: HttpRequest, data: Data<AppState>, query: Query<UserFilter>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship(&data.pool, user_id, PERMISSION_LEVEL_FRONT).await?;

    let front = crate::database::front::get_current_front_entries(&data.pool, user_id).await.map_err(to_web_error)?;
    ok(front)
}