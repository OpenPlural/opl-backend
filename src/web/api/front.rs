use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::friend::PERMISSION_LEVEL_FRONT;
use crate::model::front::{FrontEntry, FrontEntryId, ViewedFrontEntry};
use crate::model::user::UserFilter;
use crate::model::{IdResponse, PageQuery};
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, put, HttpRequest};
use crate::error::WebError;

#[get("/")]
pub async fn get_front_entries(req: HttpRequest, data: Data<AppState>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_FRONT).await?;

    let front = crate::database::front::get_current_front_entries(&data.pool, user_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)?;
    if token.user_id != user_id {
        ok(front.into_iter().map(Into::into).collect::<Vec<ViewedFrontEntry>>())
    } else {
        ok(front)
    }
}

#[get("/{id}")]
pub async fn get_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<FrontEntryId>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_FRONT).await?;

    let field_id = path.into_inner();
    if let Some(front) = crate::database::front::get_front_entry_by_id(&data.pool, field_id, user_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)? {
        if token.user_id != user_id {
            if front.ended_at.is_some() {
                return not_found();
            }
            ok(ViewedFrontEntry::from(front))
        } else {
            ok(front)
        }
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn add_front_entry(req: HttpRequest, data: Data<AppState>, body: Json<FrontEntry>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    if crate::database::front::is_fronting(&data.pool, token.user_id, body.member_id).await.map_err(to_web_error)? {
        return Err(WebError::AlreadyFronting);
    }

    let id = crate::database::front::add_front_entry(&data.pool, &body).await.map_err(to_web_error)?;
    crate::frontwatch::notify_front_change(token.user_id).await;
    ok(IdResponse {
        id
    })
}

#[delete("/{id}")]
pub async fn delete_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<FrontEntryId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let entry_id = path.into_inner();
    crate::database::front::delete_front_entry(&data.pool, entry_id, token.user_id).await.map_err(to_web_error)?;
    crate::frontwatch::notify_front_change(token.user_id).await;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<FrontEntryId>, body: Json<FrontEntry>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let entry_id = path.into_inner();
    body.id = entry_id;
    body.user_id = token.user_id;

    if let Some(active_entry) = crate::database::front::get_active_front_entry_by_member(&data.pool, token.user_id, body.member_id).await.map_err(to_web_error)? {
        if active_entry.id != entry_id {
            return Err(WebError::AlreadyFronting);
        }
    }

    crate::database::front::edit_front_entry(&data.pool, &body).await.map_err(to_web_error)?;
    crate::frontwatch::notify_front_change(token.user_id).await;
    ok_none()
}

#[get("/history")]
pub async fn get_front_history(req: HttpRequest, data: Data<AppState>, query: Query<PageQuery>) -> WebResult {
    let token = get_token(&req).unwrap();

    let front = crate::database::front::get_front_history(&data.pool, token.user_id, query.page).await.map_err(to_web_error)?;
    ok(front)
}