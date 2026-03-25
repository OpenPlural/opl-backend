use crate::database::session::check_session;
use crate::database::{to_web_error, DatabasePool};
use crate::error::WebError;
use crate::model::user::UserId;
use crate::AppState;
use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub type TokenId = i32;

#[derive(Debug, Clone)]
pub struct RequestToken {
    pub token_id: TokenId,
    pub user_id: UserId,
    pub write: bool,
    pub admin: bool, // Admin tokens are user sessions obtained by login, as opposed to tokens generated for API access
}

impl RequestToken {
    pub fn require_write(&self) -> Result<(), WebError> {
        if self.write {
            Ok(())
        } else {
            Err(WebError::TokenPermissionDeniedWrite)
        }
    }

    pub fn require_admin(&self) -> Result<(), WebError> {
        if self.admin {
            Ok(())
        } else {
            Err(WebError::TokenPermissionDeniedAdmin)
        }
    }

    pub fn require_self(&self, user_id: UserId) -> Result<(), WebError> {
        if self.user_id == user_id {
            Ok(())
        } else {
            Err(WebError::TokenPermissionDeniedSelf)
        }
    }

    pub async fn check_friendship(&self, pool: &DatabasePool, user_id: UserId, permission_level: i8) -> Result<(), WebError> {
        if user_id == self.user_id {
            return Ok(());
        }
        let settings = crate::database::friend::get_friend_settings(pool, user_id, self.user_id).await.map_err(to_web_error)?;
        if settings.check_permission(permission_level) {
            Ok(())
        } else {
            Err(WebError::FriendPermissionDenied)
        }
    }
}

pub async fn bearer_validation(req: ServiceRequest, bearer: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = bearer.token();
    let data = req.app_data::<Data<AppState>>().unwrap();
    match check_session(&data.pool, &token).await {
        Ok(Some(token)) => {
            req.extensions_mut().insert(token);
            Ok(req)
        }
        Ok(None) => {
            Err((WebError::InvalidToken.into(), req))
        }
        Err(err) => {
            Err((WebError::DatabaseError(err).into(), req))
        }
    }
}

pub fn get_token(req: &HttpRequest) -> Option<RequestToken> {
    req.extensions().get::<RequestToken>().cloned()
}