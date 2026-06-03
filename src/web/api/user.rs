use crate::database::to_web_error;
use crate::middleware::{get_token, RequestToken};
use crate::model::friend::{PERMISSION_LEVEL_FRONT, PERMISSION_LEVEL_MEMBERS};
use crate::model::front::ViewedFrontEntry;
use crate::model::user::{ExtendedUserInfo, UserId, UserInfo};
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, patch, HttpRequest};

#[get("/self")]
pub async fn get_self_user(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();

    if let Some((user, _)) = crate::database::user::get_user_by_id(&data.pool, token.user_id, true).await.map_err(to_web_error)? {
        ok(user)
    } else {
        not_found()
    }
}

#[get("/{id}")]
pub async fn get_user(req: HttpRequest, data: Data<AppState>, path: Path<UserId>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    let user_id = path.into_inner();
    let permission_level = token.check_friendship(&data.pool, user_id).await?;
    let friend_viewer = token.as_friend_viewer(user_id);

    if let Some((user, _)) = crate::database::user::get_user_by_id(&data.pool, user_id, token.user_id == user_id).await.map_err(to_web_error)? {
        let (folders, members, front) = if permission_level >= PERMISSION_LEVEL_MEMBERS {
            let folders = crate::database::folder::get_folders(&data.pool, user_id, friend_viewer).await.map_err(to_web_error)?;
            let members = crate::database::member::get_members(&data.pool, user_id, friend_viewer).await.map_err(to_web_error)?;
            let front = if permission_level >= PERMISSION_LEVEL_FRONT {
                let front = crate::database::front::get_current_front_entries(&data.pool, user_id, friend_viewer).await.map_err(to_web_error)?;
                let front: Vec<ViewedFrontEntry> = front.into_iter().map(Into::into).collect();
                Some(front)
            } else {
                None
            };

            let folders = folders.into_iter().map(Into::into).collect();
            let members = members.into_iter()
                .filter(|m| {
                    if !m.custom {
                        return true;
                    }
                    if let Some(front) = &front && front.iter().find(|f| f.member_id == m.id).is_some() {
                        return true;
                    }
                    false
                })
                .map(Into::into)
                .collect();

            (Some(folders), Some(members), front)
        } else {
            (None, None, None)
        };
        ok(ExtendedUserInfo {
            user,
            folders,
            members,
            front,
        })
    } else {
        not_found()
    }
}

#[get("/{id}/name")]
pub async fn get_username(req: HttpRequest, data: Data<AppState>, path: Path<UserId>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    let user_id = path.into_inner();
    token.check_friendship(&data.pool, user_id).await?;

    let username = crate::database::user::get_username(&data.pool, user_id).await.map_err(to_web_error)?;
    if let Some(username) = username {
        ok(username)
    } else {
        not_found()
    }
}

#[patch("/self")]
pub async fn edit_user(req: HttpRequest, data: Data<AppState>, body: Json<UserInfo>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.id = token.user_id;
    
    crate::database::user::update_user(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}