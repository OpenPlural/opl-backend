use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::web::{ok, ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpRequest};
use crate::error::WebError;
use crate::model::pluralkit::PkConfig;

#[get("/")]
pub async fn get_pk_config(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_session()?;

    let config = crate::database::pluralkit::get_pluralkit_config(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let config = config.unwrap_or_default();
    ok(config)
}

#[post("/")]
pub async fn update_pk_config(req: HttpRequest, data: Data<AppState>, body: Json<PkConfig>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_session()?;

    let config = body.into_inner();

    crate::database::pluralkit::setup_pluralkit(&data.pool, token.user_id, &config).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/sync")]
pub async fn sync_pk(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_session()?;

    let user_id = token.user_id;
    let config = crate::database::pluralkit::get_pluralkit_config(&data.pool, user_id).await.map_err(to_web_error)?;
    let config = config.unwrap_or_default();

    let Some(token) = config.token else { return Err(WebError::PluralKitNotConfigured) };

    let members = crate::database::member::get_members(&data.pool, user_id, None).await.map_err(to_web_error)?;
    
    crate::pluralkit::sync(&data.pool, &token, config.display_name, members).await?;

    ok_none()
}