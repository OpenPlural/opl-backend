use crate::database::{DatabasePool, DatabaseResult};
use crate::model::folder::{Folder, FolderId};
use crate::model::user::UserId;
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_folders(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<Folder>> {
    let folders = query("SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(folders.into_iter().map(folder).collect())
}

pub async fn get_folder_by_id(pool: &DatabasePool, folder_id: FolderId, user_id: UserId) -> DatabaseResult<Option<Folder>> {
    let res = query("SELECT ID, UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE ID = ? AND UserId = ?")
        .bind(folder_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(res.map(folder))
}

pub async fn create_folder(pool: &DatabasePool, folder: &Folder) -> DatabaseResult<()> {
    query("INSERT INTO Folder (ID, UserId, ParentId, Name, Description, Emoji, Color) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(folder.id)
        .bind(folder.user_id)
        .bind(folder.parent_id)
        .bind(&folder.name)
        .bind(&folder.description)
        .bind(&folder.emoji)
        .bind(folder.color)
        .execute(pool.as_ref())
        .await?;

    Ok(())
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