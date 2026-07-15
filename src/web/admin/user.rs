use crate::database::to_web_error;
use crate::model::admin::AdminUserInfo;
use crate::model::user::UserId;
use crate::web::admin::verify_admin_token;
use crate::web::{not_found, ok, ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Path};
use actix_web::{get, post, HttpRequest};
use crate::web::api::export::do_export;

#[get("/")]
pub async fn get_all_users(req: HttpRequest, data: Data<AppState>) -> WebResult {
    verify_admin_token(&req)?;

    let users = crate::database::admin::get_users(&data.pool).await.map_err(to_web_error)?;
    ok(users)
}

#[get("/{id}")]
pub async fn get_user_by_id(req: HttpRequest, data: Data<AppState>, id: Path<UserId>) -> WebResult {
    verify_admin_token(&req)?;

    let user_id = id.into_inner();

    if let Some((user, friend_code)) = crate::database::user::get_user_by_id(&data.pool, user_id, true).await.map_err(to_web_error)? {
        let disabled = crate::database::user::is_disabled(&data.pool, user_id).await.map_err(to_web_error)?;
        let can_reset_password = crate::database::user::can_reset_password(&data.pool, user_id).await.map_err(to_web_error)?;
        let created_at = crate::database::user::get_user_creation_date(&data.pool, user_id).await.map_err(to_web_error)?;
        let folders = crate::database::folder::get_folders(&data.pool, user_id, None).await.map_err(to_web_error)?;
        let members = crate::database::member::get_members(&data.pool, user_id, None).await.map_err(to_web_error)?;
        let front = crate::database::front::get_current_front_entries(&data.pool, user_id, None).await.map_err(to_web_error)?;

        ok(AdminUserInfo {
            created_at: created_at.unwrap(),
            disabled: disabled.unwrap(),
            can_reset_password: can_reset_password.unwrap(),
            friend_code,
            user,
            folders,
            members,
            front,
        })
    } else {
        not_found()
    }
}

#[post("/{id}/disable")]
pub async fn disable_user(req: HttpRequest, data: Data<AppState>, id: Path<UserId>) -> WebResult {
    verify_admin_token(&req)?;

    let user_id = id.into_inner();
    crate::database::user::set_disabled(&data.pool, user_id, true).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/{id}/enable")]
pub async fn enable_user(req: HttpRequest, data: Data<AppState>, id: Path<UserId>) -> WebResult {
    verify_admin_token(&req)?;

    let user_id = id.into_inner();
    crate::database::user::set_disabled(&data.pool, user_id, false).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/{id}/export")]
pub async fn export_user(req: HttpRequest, data: Data<AppState>, id: Path<UserId>) -> WebResult {
    verify_admin_token(&req)?;

    let user_id = id.into_inner();
    do_export(data, user_id).await
}