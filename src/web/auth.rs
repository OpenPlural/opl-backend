use crate::database::to_web_error;
use crate::error::WebError;
use crate::model::auth::{ChangePasswordRequest, DeleteRequest, LoginRequest, RegisterRequest};
use crate::web::{ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json};
use actix_web::post;

#[post("/register")]
pub async fn register(req: Json<RegisterRequest>, data: Data<AppState>) -> WebResult {
    if let Ok(dr) = std::env::var("DISABLE_REGISTRATION") && let Ok (dr) = dr.parse::<bool>() && dr {
        return Err(WebError::RegistrationDisabled);
    }

    let req = req.into_inner();
    req.validate().map_err(validation_error)?;

    if crate::database::user::register(&data.pool, &req.name, &req.password, req.system).await.map_err(to_web_error)? {
        ok_none()
    } else {
        Err(WebError::UsernameAlreadyExists)
    }
}

#[post("/login")]
pub async fn login(req: Json<LoginRequest>, data: Data<AppState>) -> WebResult {
    let req = req.into_inner();
    req.validate().map_err(validation_error)?;

    if let Some(user) = crate::database::user::login(&data.pool, &req.device, &req.name, &req.password).await.map_err(to_web_error)? {
        ok(user)
    } else {
        Err(WebError::InvalidCredentials)
    }
}

#[post("/delete-account")]
pub async fn delete_account(req: Json<DeleteRequest>, data: Data<AppState>) -> WebResult {
    let req = req.into_inner();
    req.validate().map_err(validation_error)?;

    if crate::database::user::delete(&data.pool, req.id, &req.password).await.map_err(to_web_error)? {
        ok_none()
    } else {
        Err(WebError::InvalidCredentials)
    }
}

#[post("/change-password")]
pub async fn change_password(req: Json<ChangePasswordRequest>, data: Data<AppState>) -> WebResult {
    let req = req.into_inner();
    req.validate().map_err(validation_error)?;

    if crate::database::user::change_password(&data.pool, req.id, &req.old_password, &req.new_password).await.map_err(to_web_error)? {
        ok_none()
    } else {
        Err(WebError::InvalidCredentials)
    }
}