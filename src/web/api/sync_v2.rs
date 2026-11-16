use actix_web::{post, HttpRequest};
use actix_web::web::{Data, Json};
use crate::AppState;
use crate::database::session::extend_session;
use crate::database::{to_web_error, DatabasePool, DatabaseResult, DatabaseConnection, DatabaseTransaction};
use crate::middleware::{get_token, RequestToken};
use crate::model::front::{FrontEntry, FrontEntryId};
use crate::model::sync_v2::{FrontSyncRequest, FrontSyncResponse, SyncIds, SyncRequest, SyncResponse, SyncUpdates, Syncable};
use crate::model::user::{UserId, UserInfo};
use crate::model::Validator;
use crate::web::{not_found, ok, WebResult, validation_error};

#[post("/")]
pub async fn sync_v2(req: HttpRequest, data: Data<AppState>, body: Json<SyncRequest>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    if let Some((user, friend_code)) = crate::database::user::get_user_by_id(&data.pool, token.user_id, true).await.map_err(to_web_error)? {
        if let Some(session_id) = token.session_id {
            extend_session(&data.pool, session_id).await.map_err(to_web_error)?;
        }

        let body = body.into_inner();
        body.validate().map_err(validation_error)?;

        let resp = sync_all(&data.pool, user, friend_code, body).await.map_err(to_web_error)?;
        ok(resp)
    } else {
        not_found()
    }
}

#[post("/front")]
pub async fn sync_front(req: HttpRequest, data: Data<AppState>, body: Json<FrontSyncRequest>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let (assign_front, front) = do_sync_front(&data.pool, None, token.user_id, body.front).await.map_err(to_web_error)?;
    ok(FrontSyncResponse {
        assign_front,
        front,
    })
}

async fn do_sync_front(pool: &DatabasePool, tx: Option<DatabaseTransaction<'_>>, user_id: UserId, local_front: Vec<FrontEntry>) -> DatabaseResult<(Vec<FrontEntryId>, Vec<FrontEntry>)> {
    let mut transaction = if let Some(tx) = tx {
        tx
    } else {
        pool.begin().await?
    };
    let mut created_front_ids = Vec::with_capacity(local_front.len());
    let mut server_fronts = crate::database::front::get_current_front_entries(&pool, user_id, None).await?;
    for mut local_front in local_front {
        local_front.user_id = user_id;
        if local_front.id == 0 {
            let id = crate::database::front::add_front_entry(transaction.as_mut(), &local_front).await?;
            local_front.id = id;
            println!("create front {local_front:?} at {id}");
            created_front_ids.push(id);
            server_fronts.push(local_front);
        } else if let Some(server_front) = server_fronts.iter_mut().find(|f| f.id == local_front.id) {
            if local_front.updated_at >= server_front.updated_at {
                crate::database::front::edit_front_entry(transaction.as_mut(), &local_front).await?;
                *server_front = local_front;
            }
        }
    }
    transaction.commit().await?;

    Ok((created_front_ids, server_fronts))
}

macro_rules! sync_resource {
    ($pool:expr, $user:expr, $body:expr, $tx:expr, $field:ident, $db:ident, $ids:ident, $updates:ident, $create:ident, $update:ident, $delete:ident) => {
        sync(
            $user,
            $body.known.$field,
            $body.deleted.$field,
            $body.updated.$field,
            crate::database::$db::$ids($pool, $user).await?,
            crate::database::$db::$updates($pool, $user, &$body.time).await?,
            async |tx, t| crate::database::$db::$create(tx, t).await,
            async |tx, t| crate::database::$db::$update(tx, t).await,
            async |tx, ids| crate::database::$db::$delete(tx, ids, $user).await,
            $tx,
        ).await?
    };
}

async fn sync_all(pool: &DatabasePool, user: UserInfo, friend_code: String, body: SyncRequest) -> DatabaseResult<SyncResponse> {
    let time = crate::database::time::get_database_time(pool).await?;

    let mut transaction = pool.begin().await?;
    let (created_folders, updated_folders, deleted_folders) = sync_resource!(pool, user.id, body, transaction.as_mut(), folders, folder, get_folder_ids, get_updated_folders, create_folder, edit_folder, delete_folders);
    let (created_members, updated_members, deleted_members) = sync_resource!(pool, user.id, body, transaction.as_mut(), members, member, get_member_ids, get_updated_members, create_member_with_folders, edit_member_with_folders, delete_members);
    let (created_fields, updated_fields, deleted_fields) = sync_resource!(pool, user.id, body, transaction.as_mut(), fields, fields, get_field_ids, get_updated_fields, create_field, edit_field, delete_fields);
    let (created_field_values, updated_field_values, deleted_field_values) = sync_resource!(pool, user.id, body, transaction.as_mut(), field_values, fields, get_field_value_ids, get_updated_field_values, create_field_value, edit_field_value, clear_field_values);
    let (created_polls, updated_polls, deleted_polls) = sync_resource!(pool, user.id, body, transaction.as_mut(), polls, poll, get_poll_ids, get_updated_polls, create_poll, edit_poll, delete_polls);
    let (created_poll_answers, updated_poll_answers, deleted_poll_answers) = sync_resource!(pool, user.id, body, transaction.as_mut(), poll_answers, poll, get_poll_answer_ids, get_updated_poll_answers, create_poll_answer, edit_poll_answer, delete_poll_answers);
    let (created_photo_albums, updated_photo_albums, deleted_photo_albums) = sync_resource!(pool, user.id, body, transaction.as_mut(), photo_albums, gallery, get_photo_album_ids, get_updated_photo_albums, create_photo_album, edit_photo_album, delete_photo_albums);

    let (created_front, server_front) = do_sync_front(pool, Some(transaction), user.id, body.front).await?;

    Ok(SyncResponse {
        time,
        user,
        friend_code,
        delete: SyncIds {
            folders: deleted_folders,
            members: deleted_members,
            fields: deleted_fields,
            field_values: deleted_field_values,
            polls: deleted_polls,
            poll_answers: deleted_poll_answers,
            photo_albums: deleted_photo_albums,
        },
        assign: SyncIds {
            folders: created_folders,
            members: created_members,
            fields: created_fields,
            field_values: created_field_values,
            polls: created_polls,
            poll_answers: created_poll_answers,
            photo_albums: created_photo_albums,
        },
        update: SyncUpdates {
            folders: updated_folders,
            members: updated_members,
            fields: updated_fields,
            field_values: updated_field_values,
            polls: updated_polls,
            poll_answers: updated_poll_answers,
            photo_albums: updated_photo_albums,
        },
        assign_front: created_front,
        front: server_front,
    })
}

async fn sync<T: Syncable, CreFn, UpdFn, DelFn>(
    user_id: UserId,
    local_known: Vec<i64>,
    mut local_deleted: Vec<i64>,
    local_updated: Vec<T>,
    mut remote_known: Vec<i64>,
    mut remote_updated: Vec<T>,
    mut create_fn: CreFn,
    mut update_fn: UpdFn,
    delete_fn: DelFn,
    tx: &mut DatabaseConnection,
) -> DatabaseResult<(Vec<i64>, Vec<T>, Vec<i64>)>
where
    CreFn: AsyncFnMut(&mut DatabaseConnection, &T) -> DatabaseResult<i64>,
    UpdFn: AsyncFnMut(&mut DatabaseConnection, &T) -> DatabaseResult<()>,
    DelFn: AsyncFnOnce(&mut DatabaseConnection, Vec<i64>) -> DatabaseResult<()>,
{
    let remote_update_ids: Vec<i64> = remote_updated.iter().map(|t| t.id()).collect();
    local_deleted.retain(|id| !remote_update_ids.contains(id));
    remote_known.retain(|id| !local_deleted.contains(id));
    delete_fn(tx, local_deleted).await?;

    let push_delete = local_known.iter().filter(|id| !remote_known.contains(id) && local_updated.iter().find(|t| t.id() == **id).is_none()).cloned().collect();
    let mut create_ids = Vec::with_capacity(local_updated.len());
    let mut update_ids = Vec::with_capacity(local_updated.len());
    for mut updated in local_updated {
        updated.set_user(user_id);
        let id = updated.id();
        if id == 0 {
            let id = create_fn(tx, &updated).await?;
            create_ids.push(id);
        } else if let Some(server_update) = remote_updated.iter().find(|t| t.id() == id) {
            let local_time = updated.updated_at();
            let remote_time = server_update.updated_at();
            if local_time >= remote_time {
                update_fn(tx, &updated).await?;
                update_ids.push(id);
            }
        } else {
            update_fn(tx, &updated).await?;
            update_ids.push(id);
        }
    }

    remote_updated.retain(|t| !update_ids.contains(&t.id()));

    Ok((create_ids, remote_updated, push_delete))
}