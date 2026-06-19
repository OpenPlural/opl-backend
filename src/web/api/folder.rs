use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::folder::{Folder, FolderId, ViewedFolder};
use crate::model::friend::PERMISSION_LEVEL_MEMBERS;
use crate::model::user::UserFilter;
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, put, HttpRequest};
use crate::model::IdResponse;

#[get("/")]
pub async fn get_folders(req: HttpRequest, data: Data<AppState>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let folders = crate::database::folder::get_folders(&data.pool, user_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)?;
    if token.user_id != user_id {
        ok(folders.into_iter().map(Into::into).collect::<Vec<ViewedFolder>>())
    } else {
        ok(folders)
    }
}

#[get("/{id}")]
pub async fn get_folder(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let folder_id = path.into_inner();

    if let Some(folder) = crate::database::folder::get_folder_by_id(&data.pool, folder_id, user_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)? {
        if token.user_id != user_id {
            ok(ViewedFolder::from(folder))
        } else {
            ok(folder)
        }
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn create_folder(req: HttpRequest, data: Data<AppState>, body: Json<Folder>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;
    
    let id = crate::database::folder::create_folder(&*data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
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

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let folder_id = path.into_inner();
    body.id = folder_id;
    body.user_id = token.user_id;
    
    crate::database::folder::edit_folder(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/privacy")]
pub async fn get_folder_privacy(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let folder_id = path.into_inner();

    let buckets = crate::database::privacy::get_folder_privacy_buckets(&data.pool, folder_id, token.user_id).await.map_err(to_web_error)?;
    ok(buckets)
}