use crate::database::session::check_session;
use crate::database::{to_web_error, DatabasePool};
use crate::error::WebError;
use crate::model::user::UserId;
use crate::AppState;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest};
use actix_web::body::MessageBody;
use actix_web::cookie::{Cookie, CookieBuilder, SameSite};
use actix_web::middleware::Next;
use crate::database::apikey::check_api_key;
use crate::model::session::SessionId;
use crate::security::sha256;

pub const SESSION_COOKIE_NAME: &'static str = "Session";
#[cfg(debug_assertions)]
pub const SESSION_COOKIE_DOMAIN: &'static str = "localhost";
#[cfg(not(debug_assertions))]
pub const SESSION_COOKIE_DOMAIN: &'static str = env!("SESSION_COOKIE_DOMAIN");
#[cfg(debug_assertions)]
pub const SESSION_COOKIE_SAME_SITE_POLICY: SameSite = SameSite::None;
#[cfg(not(debug_assertions))]
pub const SESSION_COOKIE_SAME_SITE_POLICY: SameSite = SameSite::Strict;

#[derive(Debug, Clone)]
pub struct RequestToken {
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub write: bool,
}

impl RequestToken {
    pub fn require_write(&self) -> Result<(), WebError> {
        if self.write {
            Ok(())
        } else {
            Err(WebError::TokenPermissionDeniedWrite)
        }
    }

    pub fn require_session(&self) -> Result<(), WebError> {
        if self.session_id.is_some() {
            Ok(())
        } else {
            Err(WebError::TokenPermissionDeniedAdmin)
        }
    }

    pub fn as_friend_viewer(&self, user_id: UserId) -> Option<UserId> {
        if self.user_id == user_id {
            None
        } else {
            Some(self.user_id)
        }
    }

    pub async fn check_friendship(&self, pool: &DatabasePool, user_id: UserId) -> Result<i8, WebError> {
        if user_id == self.user_id {
            return Ok(i8::MAX);
        }
        let settings = crate::database::friend::get_friend_settings(pool, user_id, self.user_id).await.map_err(to_web_error)?;
        if let Some(settings) = settings {
            Ok(settings.permission_level)
        } else if crate::database::friend::check_friendship(pool, user_id, self.user_id).await.map_err(to_web_error)? {
            Ok(0)
        } else {
            Err(WebError::FriendPermissionDenied)
        }
    }

    pub async fn check_friendship_permissions(&self, pool: &DatabasePool, user_id: UserId, permission_level: i8) -> Result<(), WebError> {
        let friend_permission_level = self.check_friendship(pool, user_id).await?;
        if friend_permission_level >= permission_level {
            Ok(())
        } else {
            Err(WebError::FriendPermissionDenied)
        }
    }
}

pub async fn authenticator_mw(req: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    try_authenticate(&req).await?;
    next.call(req).await
}

async fn try_authenticate(req: &ServiceRequest) -> Result<(), Error> {
    let data = req.app_data::<Data<AppState>>().unwrap();
    let result = if let Some(cookie) = req.cookie(SESSION_COOKIE_NAME) {
        let token = cookie.value();
        let hashed_token = sha256(token);
        check_session(&data.pool, &hashed_token).await
    } else if let Some(authorization) = req.headers().get("Authorization") && let Some(token) =
            authorization.to_str().ok().map(|token| token.strip_prefix("Bearer ")).flatten() {
        let hashed_token = sha256(token);
        check_api_key(&data.pool, &hashed_token).await
    } else {
        Ok(None)
    };
    match result {
        Ok(Some(token)) => {
            req.extensions_mut().insert(token);
            Ok(())
        }
        Ok(None) => {
            Err(WebError::InvalidToken.into())
        }
        Err(err) => {
            Err(WebError::DatabaseError(err).into())
        }
    }
}

pub fn get_token(req: &HttpRequest) -> Option<RequestToken> {
    req.extensions().get::<RequestToken>().cloned()
}

pub fn make_session_cookie<'a>(token: String) -> Cookie<'a> {
    CookieBuilder::new(SESSION_COOKIE_NAME, token)
        .domain(SESSION_COOKIE_DOMAIN)
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SESSION_COOKIE_SAME_SITE_POLICY)
        .permanent()
        .finish()
}