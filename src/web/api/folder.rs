use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::folder::{Folder, FolderId};
use crate::model::friend::PERMISSION_LEVEL_MEMBERS;
use crate::model::user::UserFilter;
use crate::web::{not_found, ok, ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, put, HttpRequest};
use crate::error::WebError;

#[get("/")]
pub async fn get_folders(req: HttpRequest, data: Data<AppState>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let folders = crate::database::folder::get_folders(&data.pool, user_id).await.map_err(to_web_error)?;
    ok(folders)
}

#[get("/{id}")]
pub async fn get_folder(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let folder_id = path.into_inner();

    if let Some(folder) = crate::database::folder::get_folder_by_id(&data.pool, folder_id, user_id).await.map_err(to_web_error)? {
        ok(folder)
    } else {
        not_found()
    }
}

#[put("/{id}")]
pub async fn create_folder(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>, body: Json<Folder>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let folder_id = path.into_inner();
    if crate::database::folder::get_folder_by_id(&data.pool, folder_id, token.user_id).await.map_err(to_web_error)?.is_some() {
        return Err(WebError::IdDuplicate);
    }

    let mut body = body.into_inner();
    if body.id != folder_id {
        return Err(WebError::IdMismatch);
    }
    body.user_id = token.user_id;
    crate::database::folder::create_folder(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/{id}")]
pub async fn delete_folder(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let folder_id = path.into_inner();
    crate::database::folder::delete_folder(&data.pool, folder_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_folder(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>, body: Json<Folder>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let folder_id = path.into_inner();

    let mut body = body.into_inner();
    if body.id != folder_id {
        return Err(WebError::IdMismatch);
    }
    body.user_id = token.user_id;
    crate::database::folder::edit_folder(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}