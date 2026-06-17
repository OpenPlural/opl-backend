use crate::database::to_web_error;
use crate::middleware::{get_token, RequestToken};
use crate::web::{ok, ok_none, ok_none_unauth, WebResult};
use crate::AppState;
use actix_web::web::{Data, Path};
use actix_web::{delete, get, HttpRequest};
use crate::model::session::SessionId;

#[get("/")]
pub async fn get_sessions(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let sessions = crate::database::session::get_sessions(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(sessions)
}

#[delete("/self")]
pub async fn invalidate_current_session(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    crate::database::session::delete_session(&data.pool, token.user_id, token.session_id.unwrap()).await.map_err(to_web_error)?;
    ok_none_unauth()
}

#[delete("/{id}")]
pub async fn invalidate_session(req: HttpRequest, data: Data<AppState>, path: Path<SessionId>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let token_id = path.into_inner();
    crate::database::session::delete_session(&data.pool, token.user_id, token_id).await.map_err(to_web_error)?;
    if token_id == token.session_id.unwrap() {
        ok_none_unauth()
    } else {
        ok_none()
    }
}