use chrono::{DateTime, Utc};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::folder::{Folder, FolderId};
use crate::model::user::UserId;
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_folder_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FolderId>> {
    let ids = query("SELECT ID FROM Folder WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_updated_folders(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<Folder>> {
    let updated = query("SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(updated.into_iter().map(folder).collect())
}

pub async fn get_folders(pool: &DatabasePool, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<Vec<Folder>> {
    let folders = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt
FROM Folder f
WHERE UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketFolder pfo
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pfo.BucketId AND pf.UserId = pfo.UserId
             WHERE pfo.FolderId = f.ID AND pf.FriendId = ?
)
"#)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_all(pool.as_ref())
            .await?
    } else {
        query("SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE UserId = ?")
            .bind(user_id)
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(folders.into_iter().map(folder).collect())
}

pub async fn get_folder_by_id(pool: &DatabasePool, folder_id: FolderId, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<Option<Folder>> {
    let res = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt
FROM Folder f
WHERE ID = ? AND UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketFolder pfo
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pfo.BucketId AND pf.UserId = pfo.UserId
             WHERE pfo.FolderId = f.ID AND pf.FriendId = ?
)
"#)
            .bind(folder_id)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_optional(pool.as_ref())
            .await?
    } else {
        query("SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE ID = ? AND UserId = ?")
            .bind(folder_id)
            .bind(user_id)
            .fetch_optional(pool.as_ref())
            .await?
    };

    Ok(res.map(folder))
}

pub async fn get_folders_by_ids(pool: &DatabasePool, folder_ids: &Vec<FolderId>, user_id: UserId) -> DatabaseResult<Vec<Folder>> {
    if folder_ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = folder_ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
    let sql = format!("SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE ID IN ({placeholders}) AND UserId = ?");
    let mut query = query(sql.as_str());
    for id in folder_ids {
        query = query.bind(id);
    }
    let folders = query
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(folders.into_iter().map(folder).collect())
}

pub async fn create_folder(pool: &DatabasePool, folder: &Folder) -> DatabaseResult<FolderId> {
    let id = query("INSERT INTO Folder (UserId, ParentId, Name, Description, Emoji, Color) VALUES (?, ?, ?, ?, ?, ?) RETURNING ID")
        .bind(folder.user_id)
        .bind(folder.parent_id)
        .bind(&folder.name)
        .bind(&folder.description)
        .bind(&folder.emoji)
        .bind(folder.color)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn delete_folder(pool: &DatabasePool, folder_id: FolderId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM Folder WHERE ID = ? AND UserId = ?")
        .bind(folder_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_folder(pool: &DatabasePool, folder: &Folder) -> DatabaseResult<()> {
    query("UPDATE Folder SET ParentId = ?, Name = ?, Description = ?, Emoji = ?, Color = ? WHERE ID = ? AND UserId = ?")
        .bind(folder.parent_id)
        .bind(&folder.name)
        .bind(&folder.description)
        .bind(&folder.emoji)
        .bind(&folder.color)
        .bind(folder.id)
        .bind(folder.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

fn folder(row: MySqlRow) -> Folder {
    let id = row.get("ID");
    let user_id = row.get("UserId");
    let parent_id = row.get("ParentId");
    let name = row.get("Name");
    let description = row.get("Description");
    let emoji = row.get("Emoji");
    let color = row.get("Color");
    let created_at = row.get("CreatedAt");
    let updated_at = row.get("UpdatedAt");

    Folder {
        id,
        user_id,
        parent_id,
        name,
        description,
        emoji,
        color,
        created_at,
        updated_at,
    }
}