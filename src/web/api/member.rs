use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::folder::FolderId;
use crate::model::friend::PERMISSION_LEVEL_MEMBERS;
use crate::model::gallery::{PhotoAlbum, PhotoAlbumId};
use crate::model::member::{ExtendedViewedMember, Member, MemberId, MemberQuery, ViewedMember};
use crate::model::user::UserFilter;
use crate::model::{IdResponse, PageQuery};
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, put, HttpRequest};

#[get("/")]
pub async fn get_members(req: HttpRequest, data: Data<AppState>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let members = crate::database::member::get_members(&data.pool, user_id, Some(token.user_id)).await.map_err(to_web_error)?;
    if token.user_id != user_id {
        ok(members.into_iter().filter(|m| !m.custom).map(Into::into).collect::<Vec<ViewedMember>>())
    } else {
        ok(members)
    }
}

#[get("/{id}")]
pub async fn get_member(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, query: Query<MemberQuery>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let member_id = path.into_inner();

    if let Some(member) = crate::database::member::get_member_by_id(&data.pool, member_id, user_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)? {
        if member.custom && token.user_id != user_id {
            if !crate::database::front::is_fronting(&data.pool, user_id, member.id).await.map_err(to_web_error)? {
                return not_found();
            }
        }
        if query.extended {
            let folders = if member.custom {
                vec![]
            } else {
                let folders = crate::database::folder::get_folders_by_ids(&data.pool, &member.folders, user_id).await.map_err(to_web_error)?;
                folders.into_iter().map(Into::into).collect()
            };
            let has_gallery = crate::database::gallery::has_member_gallery(&data.pool, member.id, user_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)?;
            ok(ExtendedViewedMember {
                member: member.into(),
                folders,
                has_gallery,
            })
        } else if token.user_id != user_id {
            ok(ViewedMember::from(member))
        } else {
            ok(member)
        }
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn create_member(req: HttpRequest, data: Data<AppState>, body: Json<Member>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    let id = crate::database::member::create_member(&*data.pool, &body).await.map_err(to_web_error)?;
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

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let member_id = path.into_inner();
    body.id = member_id;
    body.user_id = token.user_id;

    crate::database::member::edit_member(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}/folders")]
pub async fn edit_member_folders(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, body: Json<Vec<FolderId>>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();
    let folder_ids = body.into_inner();
    crate::database::member::edit_member_folders(&data.pool, member_id, token.user_id, &folder_ids).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/fields/values")]
pub async fn get_member_field_values(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>) -> WebResult {
    let token = get_token(&req).unwrap();
    let member_id = path.into_inner();
    let fields = crate::database::fields::get_field_values_for_member(&data.pool, token.user_id, member_id).await.map_err(to_web_error)?;
    ok(fields)
}

#[get("/{id}/fields")]
pub async fn get_member_fields(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let member_id = path.into_inner();
    let fields = crate::database::fields::get_viewed_field_values_for_member(&data.pool, user_id, member_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)?;
    ok(fields)
}

#[get("/{id}/front-history")]
pub async fn get_member_front_history(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, query: Query<PageQuery>) -> WebResult {
    let token = get_token(&req).unwrap();

    let member_id = path.into_inner();
    let front = crate::database::front::get_front_history_of_member(&data.pool, token.user_id, member_id, query.page).await.map_err(to_web_error)?;
    ok(front)
}

#[get("/{id}/front")]
pub async fn get_member_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let member_id = path.into_inner();
    let entry = crate::database::front::get_active_front_entry_by_member(&data.pool, token.user_id, member_id).await.map_err(to_web_error)?;
    ok(entry)
}

#[get("/{memberId}/gallery")]
pub async fn get_member_gallery(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, query: Query<UserFilter>) -> WebResult {
    let token = get_token(&req).unwrap();
    let user_id = query.user_id.unwrap_or(token.user_id);
    token.check_friendship_permissions(&data.pool, user_id, PERMISSION_LEVEL_MEMBERS).await?;

    let member_id = path.into_inner();
    let albums = crate::database::gallery::get_viewed_photo_albums_for_member(&data.pool, user_id, member_id, token.as_friend_viewer(user_id)).await.map_err(to_web_error)?;
    ok(albums)
}

#[put("/{memberId}/gallery")]
pub async fn create_photo_album(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, body: Json<PhotoAlbum>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;
    body.member_id = member_id;

    let id = crate::database::gallery::create_photo_album(&*data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[delete("/{memberId}/gallery/{albumId}")]
pub async fn delete_photo_album(req: HttpRequest, data: Data<AppState>, path: Path<(MemberId, PhotoAlbumId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (member_id, album_id) = path.into_inner();
    crate::database::gallery::delete_photo_album(&data.pool, album_id, member_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/gallery/{albumId}")]
pub async fn delete_photo_album_by_id(req: HttpRequest, data: Data<AppState>, path: Path<PhotoAlbumId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let album_id = path.into_inner();
    crate::database::gallery::delete_photo_album_by_id(&data.pool, album_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{memberId}/gallery/{albumId}")]
pub async fn edit_photo_album(req: HttpRequest, data: Data<AppState>, path: Path<(MemberId, PhotoAlbumId)>, body: Json<PhotoAlbum>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let (member_id, album_id) = path.into_inner();
    body.id = album_id;
    body.user_id = token.user_id;
    body.member_id = member_id;

    crate::database::gallery::edit_photo_album(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{memberId}/gallery/{albumId}/privacy")]
pub async fn get_photo_album_privacy(req: HttpRequest, data: Data<AppState>, path: Path<(MemberId, PhotoAlbumId)>) -> WebResult {
    let token = get_token(&req).unwrap();

    let (_, album_id) = path.into_inner();

    let buckets = crate::database::privacy::get_photo_album_privacy_buckets(&data.pool, album_id, token.user_id).await.map_err(to_web_error)?;
    ok(buckets)
}

#[get("/{id}/privacy")]
pub async fn get_member_privacy(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let member_id = path.into_inner();

    let buckets = crate::database::privacy::get_member_privacy_buckets(&data.pool, member_id, token.user_id).await.map_err(to_web_error)?;
    ok(buckets)
}