use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::get_token;
use crate::model::folder::FolderId;
use crate::model::friend::PERMISSION_LEVEL_MEMBERS;
use crate::model::member::{Member, MemberId};
use crate::model::user::UserFilter;
use crate::web::{not_found, ok, ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, put, HttpRequest};
use crate::model::IdResponse;

#[get("/")]
pub async fn get_members(req: HttpRequest, data: Data<AppState>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let members = crate::database::member::get_members(&data.pool, user_id).await.map_err(to_web_error)?;
    ok(members)
}

#[get("/{id}")]
pub async fn get_member(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>) -> WebResult {
    let member_id = path.into_inner();

    if let Some(member) = crate::database::member::get_member_by_id(&data.pool, member_id).await.map_err(to_web_error)? {
        let token = get_token(&req).unwrap();
        token.check_friendship(&data.pool, member.user_id, PERMISSION_LEVEL_MEMBERS).await?;

        ok(member)
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn create_member(req: HttpRequest, data: Data<AppState>, body: Json<Member>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let body = body.into_inner();
    let id = crate::database::member::create_member(&data.pool, token.user_id, &body).await.map_err(to_web_error)?;
    if !body.folders.is_empty() {
        crate::database::member::edit_member_folders(&data.pool, id, &body.folders).await.map_err(to_web_error)?;
    }
    ok(IdResponse {
        id
    })
}

#[delete("/{id}")]
pub async fn delete_member(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();
    crate::database::member::delete_member(&data.pool, member_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_member(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, body: Json<Member>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();
    let user_id = crate::database::member::get_member_user_id(&data.pool, member_id).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    let mut body = body.into_inner();
    if body.id != member_id {
        return Err(WebError::IdMismatch);
    }
    body.user_id = user_id;
    crate::database::member::edit_member(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}/folders")]
pub async fn edit_member_folders(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, body: Json<Vec<FolderId>>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();
    let user_id = crate::database::member::get_member_user_id(&data.pool, member_id).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    let folder_ids = body.into_inner();
    crate::database::member::edit_member_folders(&data.pool, member_id, &folder_ids).await.map_err(to_web_error)?;
    ok_none()
}