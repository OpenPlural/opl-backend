use std::fmt::Debug;
use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use serde::Serialize;
use strum_macros::IntoStaticStr;
use thiserror::Error;

#[derive(Debug, Error, IntoStaticStr)]
pub enum WebError {
    #[error("Database error: {0:?}")]
    DatabaseError(anyhow::Error),
    #[error("Required field '{0}' is missing")]
    RequiredFieldMissing(&'static str),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid time format")]
    InvalidTimeFormat,
    #[error("The ID in the URL does not match the ID in the body")]
    IdMismatch,
    #[error("This ID already exists")]
    IdDuplicate,

    #[error("A user with this name already exists")]
    UsernameAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token does not have write permissions")]
    TokenPermissionDeniedWrite,
    #[error("Token does not have admin permissions (only actual sessions have admin permissions)")]
    TokenPermissionDeniedAdmin,
    #[error("You may only perform this action on yourself")]
    TokenPermissionDeniedSelf,

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

    #[error("This member is already fronting")]
    AlreadyFronting,
}

impl ResponseError for WebError {
    fn status_code(&self) -> StatusCode {
        match self {
            WebError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebError::RequiredFieldMissing(_) => StatusCode::BAD_REQUEST,
            WebError::InvalidToken => StatusCode::UNAUTHORIZED,
            WebError::InvalidTimeFormat => StatusCode::BAD_REQUEST,
            WebError::IdMismatch => StatusCode::BAD_REQUEST,
            WebError::IdDuplicate => StatusCode::CONFLICT,

            WebError::UsernameAlreadyExists => StatusCode::CONFLICT,
            WebError::InvalidCredentials => StatusCode::UNAUTHORIZED,

            WebError::TokenPermissionDeniedWrite => StatusCode::FORBIDDEN,
            WebError::TokenPermissionDeniedAdmin => StatusCode::FORBIDDEN,
            WebError::TokenPermissionDeniedSelf => StatusCode::FORBIDDEN,

            WebError::FriendPermissionDenied => StatusCode::FORBIDDEN,
            WebError::FriendRequestAlreadySent => StatusCode::CONFLICT,
            WebError::FriendRequestStillPending => StatusCode::CONFLICT,
            WebError::FriendRequestNotPending => StatusCode::CONFLICT,
            WebError::AlreadyFriends => StatusCode::CONFLICT,
            WebError::NotFriends => StatusCode::FORBIDDEN,
            WebError::InvalidFriendCode => StatusCode::NOT_FOUND,

            WebError::AlreadyFronting => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let (message, location) = match self {
            WebError::DatabaseError(err) => {
                eprintln!("Database error: {:?}", err);

                ("Database error".to_string(), Some(err.backtrace().to_string()))
            },
            err => (err.to_string(), None),
        };

        let kind: &'static str = self.into();

        HttpResponse::new(self.status_code()).set_body(BoxBody::new(serde_json::to_string(&WebErrorResponse {
            kind,
            message,
            location,
        }).unwrap_or("{}".to_string())))
    }
}

#[derive(Serialize)]
struct WebErrorResponse {
    kind: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
}