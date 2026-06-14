use crate::database::session::extend_session;
use crate::database::to_web_error;
use crate::middleware::{get_token, RequestToken};
use crate::model::sync::{SyncQuery, SyncResponse};
use crate::web::{not_found, ok, WebResult};
use crate::AppState;
use actix_web::web::{Data, Query};
use actix_web::{get, HttpRequest};
use crate::model::deletion::DeletionResourceType;

#[get("/")]
pub async fn sync(req: HttpRequest, data: Data<AppState>, query: Query<SyncQuery>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    if let Some((user, friend_code)) = crate::database::user::get_user_by_id(&data.pool, token.user_id, true).await.map_err(to_web_error)? {
        extend_session(&data.pool, token.session_id.unwrap()).await.map_err(to_web_error)?;

        let last_sync_time = query.since;
        let time = crate::database::time::get_database_time(&data.pool).await.map_err(to_web_error)?;
        let front = crate::database::front::get_current_front_entries(&data.pool, token.user_id, None).await.map_err(to_web_error)?;
        let dur = time - last_sync_time;
        if dur.num_days() < 7 {
            // send deletions
            let deletions = crate::database::deletion::get_deletions(&data.pool, token.user_id).await.map_err(to_web_error)?;
            let (folder_ids, member_ids, field_ids, field_value_ids) = deletions.into_iter()
                .fold((vec![], vec![], vec![], vec![]), |mut acc, deletion| {
                    match deletion.resource_type {
                        DeletionResourceType::Folder => acc.0.push(deletion.resource_id),
                        DeletionResourceType::Member => acc.1.push(deletion.resource_id),
                        DeletionResourceType::CustomField => acc.2.push(deletion.resource_id),
                        DeletionResourceType::CustomFieldDataValue => acc.3.push(deletion.resource_id),
                    }
                    acc
                });
            let updated_folders = crate::database::folder::get_updated_folders(&data.pool, token.user_id, &last_sync_time).await.map_err(to_web_error)?;
            let updated_members = crate::database::member::get_updated_members(&data.pool, token.user_id, &last_sync_time).await.map_err(to_web_error)?;
            let updated_fields = crate::database::fields::get_updated_fields(&data.pool, token.user_id, &last_sync_time).await.map_err(to_web_error)?;
            let updated_field_values = crate::database::fields::get_updated_field_values(&data.pool, user.id, &last_sync_time).await.map_err(to_web_error)?;

            ok(SyncResponse {
                time,
                user,
                friend_code,
                deletion_delta: true,
                folder_ids,
                member_ids,
                field_ids,
                field_value_ids,
                updated_folders,
                updated_members,
                updated_fields,
                updated_field_values,
                front,
            })
        } else {
            // send known
            let folder_ids = crate::database::folder::get_folder_ids(&data.pool, token.user_id).await.map_err(to_web_error)?;
            let member_ids = crate::database::member::get_member_ids(&data.pool, token.user_id).await.map_err(to_web_error)?;
            let field_ids = crate::database::fields::get_field_ids(&data.pool, token.user_id).await.map_err(to_web_error)?;
            let field_value_ids = crate::database::fields::get_field_value_ids(&data.pool, token.user_id).await.map_err(to_web_error)?;
            let updated_folders = crate::database::folder::get_updated_folders(&data.pool, token.user_id, &last_sync_time).await.map_err(to_web_error)?;
            let updated_members = crate::database::member::get_updated_members(&data.pool, token.user_id, &last_sync_time).await.map_err(to_web_error)?;
            let updated_fields = crate::database::fields::get_updated_fields(&data.pool, token.user_id, &last_sync_time).await.map_err(to_web_error)?;
            let updated_field_values = crate::database::fields::get_updated_field_values(&data.pool, user.id, &last_sync_time).await.map_err(to_web_error)?;

            ok(SyncResponse {
                time,
                user,
                friend_code,
                deletion_delta: false,
                folder_ids,
                member_ids,
                field_ids,
                field_value_ids,
                updated_folders,
                updated_members,
                updated_fields,
                updated_field_values,
                front,
            })
        }
    } else {
        not_found()
    }
}