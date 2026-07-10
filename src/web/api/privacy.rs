use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, patch, post, put, HttpRequest};
use crate::error::WebError;
use crate::model::fields::CustomFieldId;
use crate::model::folder::FolderId;
use crate::model::IdResponse;
use crate::model::member::MemberId;
use crate::model::privacy::{PrivacyBucket, PrivacyBucketId};
use crate::model::user::UserId;

#[get("/")]
pub async fn get_privacy_buckets(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();

    let buckets = crate::database::privacy::get_privacy_buckets(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(buckets)
}

#[get("/{id}")]
pub async fn get_privacy_bucket(req: HttpRequest, data: Data<AppState>, path: Path<PrivacyBucketId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let bucket_id = path.into_inner();

    if let Some(bucket) = crate::database::privacy::get_privacy_bucket(&data.pool, bucket_id, token.user_id).await.map_err(to_web_error)? {
        ok(bucket)
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn create_privacy_bucket(req: HttpRequest, data: Data<AppState>, body: Json<PrivacyBucket>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;
    
    let id = crate::database::privacy::create_privacy_bucket(&*data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[delete("/{id}")]
pub async fn delete_privacy_bucket(req: HttpRequest, data: Data<AppState>, path: Path<PrivacyBucketId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let bucket_id = path.into_inner();
    crate::database::privacy::delete_privacy_bucket(&data.pool, bucket_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_privacy_bucket(req: HttpRequest, data: Data<AppState>, path: Path<PrivacyBucketId>, body: Json<PrivacyBucket>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let bucket_id = path.into_inner();
    body.id = bucket_id;
    body.user_id = token.user_id;
    
    crate::database::privacy::edit_privacy_bucket(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/reorder")]
pub async fn reorder_privacy_buckets(req: HttpRequest, data: Data<AppState>, body: Json<Vec<PrivacyBucketId>>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let body = body.into_inner();
    crate::database::privacy::reorder_privacy_buckets(&data.pool, body, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[put("/{bucketId}/folder/{folderId}")]
pub async fn add_privacy_bucket_folder(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, FolderId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, folder_id) = path.into_inner();
    if let Some(bucket) = crate::database::privacy::get_simple_privacy_bucket(&data.pool, bucket_id, token.user_id).await.map_err(to_web_error)? {
        if let Some(folder_owner) = crate::database::folder::get_folder_owner(&data.pool, folder_id).await.map_err(to_web_error)? {
            if folder_owner != token.user_id {
                return Err(WebError::ResourceNotOwned);
            }
            crate::database::privacy::add_privacy_bucket_folder(&*data.pool, bucket_id, token.user_id, folder_id).await.map_err(to_web_error)?;
            return ok(bucket);
        }
    }
    not_found()
}

#[put("/{bucketId}/member/{memberId}")]
pub async fn add_privacy_bucket_member(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, MemberId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, member_id) = path.into_inner();
    if let Some(bucket) = crate::database::privacy::get_simple_privacy_bucket(&data.pool, bucket_id, token.user_id).await.map_err(to_web_error)? {
        if let Some(member_owner) = crate::database::member::get_member_owner(&data.pool, member_id).await.map_err(to_web_error)? {
            if member_owner != token.user_id {
                return Err(WebError::ResourceNotOwned);
            }
            crate::database::privacy::add_privacy_bucket_member(&*data.pool, bucket_id, token.user_id, member_id).await.map_err(to_web_error)?;
            return ok(bucket);
        }
    }
    not_found()
}

#[put("/{bucketId}/field/{fieldId}")]
pub async fn add_privacy_bucket_custom_field(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, CustomFieldId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, field_id) = path.into_inner();
    if let Some(bucket) = crate::database::privacy::get_simple_privacy_bucket(&data.pool, bucket_id, token.user_id).await.map_err(to_web_error)? {
        if let Some(field_owner) = crate::database::fields::get_field_owner(&data.pool, field_id).await.map_err(to_web_error)? {
            if field_owner != token.user_id {
                return Err(WebError::ResourceNotOwned);
            }
            crate::database::privacy::add_privacy_bucket_custom_field(&*data.pool, bucket_id, token.user_id, field_id).await.map_err(to_web_error)?;
            return ok(bucket);
        }
    }
    not_found()
}

#[put("/{bucketId}/friend/{friendId}")]
pub async fn add_privacy_bucket_friend(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, UserId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, friend_id) = path.into_inner();
    if !crate::database::friend::check_friendship(&data.pool, token.user_id, friend_id).await.map_err(to_web_error)? {
        return Err(WebError::NotFriends);
    }
    if let Some(bucket) = crate::database::privacy::get_simple_privacy_bucket(&data.pool, bucket_id, token.user_id).await.map_err(to_web_error)? {
        crate::database::privacy::add_privacy_bucket_friend(&data.pool, bucket_id, token.user_id, friend_id).await.map_err(to_web_error)?;
        return ok(bucket);
    }
    not_found()
}

#[delete("/{bucketId}/folder/{folderId}")]
pub async fn remove_privacy_bucket_folder(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, FolderId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, folder_id) = path.into_inner();
    crate::database::privacy::remove_privacy_bucket_folder(&data.pool, bucket_id, token.user_id, folder_id).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/{bucketId}/member/{memberId}")]
pub async fn remove_privacy_bucket_member(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, MemberId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, member_id) = path.into_inner();
    crate::database::privacy::remove_privacy_bucket_member(&data.pool, bucket_id, token.user_id, member_id).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/{bucketId}/field/{fieldId}")]
pub async fn remove_privacy_bucket_custom_field(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, CustomFieldId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, field_id) = path.into_inner();
    crate::database::privacy::remove_privacy_bucket_custom_field(&data.pool, bucket_id, token.user_id, field_id).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/{bucketId}/friend/{friendId}")]
pub async fn remove_privacy_bucket_friend(req: HttpRequest, data: Data<AppState>, path: Path<(PrivacyBucketId, UserId)>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let (bucket_id, friend_id) = path.into_inner();
    crate::database::privacy::remove_privacy_bucket_friend(&data.pool, bucket_id, token.user_id, friend_id).await.map_err(to_web_error)?;
    ok_none()
}