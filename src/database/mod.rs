pub mod user;
pub mod session;
pub mod friend;
pub mod member;
pub mod front;
pub mod folder;
pub mod fields;
pub mod privacy;
pub mod deletion;
pub mod time;

use std::sync::Arc;
use sqlx::MySqlPool;
use crate::error::WebError;

pub type DatabasePool = Arc<MySqlPool>;
pub type DatabaseResult<T> = Result<T, anyhow::Error>;

pub fn to_web_error(err: anyhow::Error) -> WebError {
    WebError::DatabaseError(err)
}