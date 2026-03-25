use crate::database::to_web_error;
use crate::error::WebError;
use crate::model::auth::{LoginRequest, RegisterRequest};
use crate::web::{ok, ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json};
use actix_web::post;

#[post("/register")]
pub async fn register(req: Json<RegisterRequest>, data: Data<AppState>) -> WebResult {
    let req = req.into_inner();

    if crate::database::user::register(&data.pool, &req.name, &req.password, req.system).await.map_err(to_web_error)? {
        ok_none()
    } else {
        Err(WebError::UsernameAlreadyExists)
    }
}

#[post("/login")]
pub async fn login(req: Json<LoginRequest>, data: Data<AppState>) -> WebResult {
    let req = req.into_inner();

    if let Some(user) = crate::database::user::login(&data.pool, &req.device, &req.name, &req.password).await.map_err(to_web_error)? {
        ok(user)
    } else {
        Err(WebError::InvalidCredentials)
    }
}