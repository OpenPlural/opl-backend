use crate::database::to_web_error;
use crate::middleware::{get_token, RequestToken};
use crate::model::apikey::{ApiKey, ApiKeyId};
use crate::web::{ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, put, HttpRequest};
use crate::security::{random_string, sha256, API_KEY_TOKEN_LENGTH};

#[get("/")]
pub async fn get_api_keys(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let tokens = crate::database::apikey::get_api_keys(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(tokens)
}

#[put("/")]
pub async fn create_api_key(req: HttpRequest, data: Data<AppState>, body: Json<ApiKey>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    let token = random_string(API_KEY_TOKEN_LENGTH);
    body.token = Some(token.clone());
    let token = sha256(&token);

    let id = crate::database::apikey::create_api_key(&data.pool, &body, &token).await.map_err(to_web_error)?;
    body.id = id;
    ok(body)
}

#[delete("/{id}")]
pub async fn delete_api_key(req: HttpRequest, data: Data<AppState>, path: Path<ApiKeyId>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    crate::database::apikey::delete_api_key(&data.pool, path.into_inner(), token.user_id).await.map_err(to_web_error)?;
    ok_none()
}