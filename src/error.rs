use std::fmt::Debug;
use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use serde::Serialize;
use strum_macros::IntoStaticStr;
use thiserror::Error;
use crate::middleware::make_session_cookie;

#[derive(Debug, Error, IntoStaticStr)]
pub enum WebError {
    #[error("Database error: {0:?}")]
    DatabaseError(anyhow::Error),
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Failed to set cookie {0:?}")]
    CantSetCookie(anyhow::Error),
    #[error("This web push endpoint is not trusted")]
    WebPushEndpointNotTrusted,

    #[error("Account registration is disabled")]
    RegistrationDisabled,
    #[error("A user with this name already exists")]
    UsernameAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token does not have write permissions")]
    TokenPermissionDeniedWrite,
    #[error("Token does not have admin permissions (only actual sessions have admin permissions)")]
    TokenPermissionDeniedAdmin,

    #[error("You do not have permission to perform this action on this user")]
    FriendPermissionDenied,
    #[error("A friend request has already been sent to this user")]
    FriendRequestAlreadySent,
    #[error("You have an incoming friend request from this user")]
    FriendRequestStillPending,
    #[error("You do not have a friend request from this user")]
    FriendRequestNotPending,
    #[error("You are already friends with this user")]
    AlreadyFriends,
    #[error("You are not friends with this user")]
    NotFriends,
    #[error("Invalid friend code")]
    InvalidFriendCode,
    #[error("You can not friend yourself")]
    CantFriendSelf,

    #[error("This member is already fronting")]
    AlreadyFronting,
    
    #[error("You do not own this resource")]
    ResourceNotOwned,
}

impl ResponseError for WebError {
    fn status_code(&self) -> StatusCode {
        match self {
            WebError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebError::InvalidPayload(_) => StatusCode::BAD_REQUEST,
            WebError::InvalidToken => StatusCode::UNAUTHORIZED,
            WebError::CantSetCookie(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebError::WebPushEndpointNotTrusted => StatusCode::FORBIDDEN,

            WebError::RegistrationDisabled => StatusCode::FORBIDDEN,
            WebError::UsernameAlreadyExists => StatusCode::CONFLICT,
            WebError::InvalidCredentials => StatusCode::UNAUTHORIZED,

            WebError::TokenPermissionDeniedWrite => StatusCode::FORBIDDEN,
            WebError::TokenPermissionDeniedAdmin => StatusCode::FORBIDDEN,

            WebError::FriendPermissionDenied => StatusCode::FORBIDDEN,
            WebError::FriendRequestAlreadySent => StatusCode::CONFLICT,
            WebError::FriendRequestStillPending => StatusCode::CONFLICT,
            WebError::FriendRequestNotPending => StatusCode::CONFLICT,
            WebError::AlreadyFriends => StatusCode::CONFLICT,
            WebError::NotFriends => StatusCode::FORBIDDEN,
            WebError::InvalidFriendCode => StatusCode::NOT_FOUND,
            WebError::CantFriendSelf => StatusCode::FORBIDDEN,

            WebError::AlreadyFronting => StatusCode::CONFLICT,
            
            WebError::ResourceNotOwned => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let message = match self {
            WebError::DatabaseError(err) => {
                eprintln!("Database error: {:?}", err);

                "Database error".to_string()
            },
            WebError::CantSetCookie(err) => {
                eprintln!("Cant set cookie: {:?}", err);

                "Failed to set cookie".to_string()
            }
            err => err.to_string(),
        };

        let kind: &'static str = self.into();

        let mut res = HttpResponse::new(self.status_code()).set_body(BoxBody::new(serde_json::to_string(&WebErrorResponse {
            kind,
            message,
        }).unwrap_or("{}".to_string())));
        if matches!(self, WebError::InvalidToken) {
            let cookie = make_session_cookie("".to_string());
            let _ = res.add_removal_cookie(&cookie);
        }
        res
    }
}

#[derive(Serialize)]
struct WebErrorResponse {
    kind: &'static str,
    message: String,
}