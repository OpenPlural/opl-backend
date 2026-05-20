use crate::database::friend::get_friend_ids;
use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::get_token;
use crate::model::friend::FriendSettings;
use crate::model::user::UserId;
use crate::web::{ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, patch, post, put, HttpRequest};
use uuid::Uuid;
use crate::database::front::fill_front_text;
use crate::database::user::get_users_by_ids;

#[get("/")]
pub async fn get_friends(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();

    let friends = get_friend_ids(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let friends = get_users_by_ids(&data.pool, &friends).await.map_err(to_web_error)?;
    let friends = fill_front_text(&data.pool, token.user_id, friends).await.map_err(to_web_error)?;
    ok(friends)
}

#[get("/requests/incoming")]
pub async fn get_incoming_friend_requests(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let requests = crate::database::friend::get_incoming_friend_requests(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(requests)
}

#[get("/requests/outgoing")]
pub async fn get_outgoing_friend_requests(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let requests = crate::database::friend::get_outgoing_friend_requests(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(requests)
}

#[put("/requests/{code}")]
pub async fn send_friend_request(req: HttpRequest, data: Data<AppState>, path: Path<String>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let to_friend_code = path.into_inner();
    let to_friend_code = Uuid::parse_str(&to_friend_code).map_err(|_| WebError::InvalidFriendCode)?;
    let to_user_id = crate::database::user::resolve_friend_code(&data.pool, &to_friend_code).await.map_err(to_web_error)?;
    let to_user_id = to_user_id.ok_or(WebError::InvalidFriendCode)?;

    if to_user_id == token.user_id {
        return Err(WebError::CantFriendSelf);
    }

    if crate::database::friend::check_friendship(&data.pool, token.user_id, to_user_id).await.map_err(to_web_error)? {
        return Err(WebError::AlreadyFriends)
    }
    if crate::database::friend::check_friend_request(&data.pool, token.user_id, to_user_id).await.map_err(to_web_error)? {
        return Err(WebError::FriendRequestAlreadySent)
    }
    if crate::database::friend::check_friend_request(&data.pool, to_user_id, token.user_id).await.map_err(to_web_error)? {
        return Err(WebError::FriendRequestStillPending)
    }

    crate::database::friend::send_friend_request(&data.pool, token.user_id, to_user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/requests/{code}")]
pub async fn cancel_friend_request(req: HttpRequest, data: Data<AppState>, path: Path<String>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let to_friend_code = path.into_inner();
    let to_friend_code = Uuid::parse_str(&to_friend_code).map_err(|_| WebError::InvalidFriendCode)?;
    let to_user_id = crate::database::user::resolve_friend_code(&data.pool, &to_friend_code).await.map_err(to_web_error)?;
    let to_user_id = to_user_id.ok_or(WebError::InvalidFriendCode)?;

    crate::database::friend::remove_friend_request(&data.pool, token.user_id, to_user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/requests/{code}/accept")]
pub async fn accept_friend_request(req: HttpRequest, data: Data<AppState>, path: Path<String>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let from_user_code = path.into_inner();
    let from_user_code = Uuid::parse_str(&from_user_code).map_err(|_| WebError::InvalidFriendCode)?;
    let from_user_id = crate::database::user::resolve_friend_code(&data.pool, &from_user_code).await.map_err(to_web_error)?;
    let from_user_id = from_user_id.ok_or(WebError::InvalidFriendCode)?;

    if !crate::database::friend::check_friend_request(&data.pool, from_user_id, token.user_id).await.map_err(to_web_error)? {
        return Err(WebError::FriendRequestNotPending)
    }

    crate::database::friend::accept_friend_request(&data.pool, from_user_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/requests/{code}/decline")]
pub async fn decline_friend_request(req: HttpRequest, data: Data<AppState>, path: Path<String>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let from_user_code = path.into_inner();
    let from_user_code = Uuid::parse_str(&from_user_code).map_err(|_| WebError::InvalidFriendCode)?;
    let from_user_id = crate::database::user::resolve_friend_code(&data.pool, &from_user_code).await.map_err(to_web_error)?;
    let from_user_id = from_user_id.ok_or(WebError::InvalidFriendCode)?;

    crate::database::friend::remove_friend_request(&data.pool, from_user_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/{id}")]
pub async fn unfriend(req: HttpRequest, data: Data<AppState>, path: Path<UserId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let friend_id = path.into_inner();
    crate::database::friend::remove_friend(&data.pool, token.user_id, friend_id).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/settings")]
pub async fn get_settings(req: HttpRequest, data: Data<AppState>, path: Path<UserId>) -> WebResult {
    let token = get_token(&req).unwrap();
    let friend_id = path.into_inner();
    token.check_friendship(&data.pool, friend_id).await?;

    let settings = crate::database::friend::get_friend_settings(&data.pool, token.user_id, friend_id).await.map_err(to_web_error)?.unwrap_or_default();
    ok(settings)
}

#[patch("/{id}/settings")]
pub async fn update_settings(req: HttpRequest, data: Data<AppState>, body: Json<FriendSettings>, path: Path<UserId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_admin()?;

    let friend_id = path.into_inner();

    if !crate::database::friend::check_friendship(&data.pool, token.user_id, friend_id).await.map_err(to_web_error)? {
        return Err(WebError::NotFriends)
    }

    let settings = body.into_inner();
    settings.validate().map_err(validation_error)?;
    
    crate::database::friend::update_friend_settings(&data.pool, token.user_id, friend_id, settings).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/privacy")]
pub async fn get_friend_privacy(req: HttpRequest, data: Data<AppState>, path: Path<UserId>) -> WebResult {
    let token = get_token(&req).unwrap();
    let friend_id = path.into_inner();
    token.check_friendship(&data.pool, friend_id).await?;

    let buckets = crate::database::privacy::get_friend_privacy_buckets(&data.pool, friend_id, token.user_id).await.map_err(to_web_error)?;
    ok(buckets)
}