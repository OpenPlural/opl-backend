use serde::Serialize;

pub mod auth;
pub mod folder;
pub mod member;
pub mod friend;
pub mod user;
pub mod front;

#[derive(Debug, Serialize)]
pub struct IdResponse<T: Serialize> {
    pub id: T
}