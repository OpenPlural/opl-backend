use crate::database::{DatabasePool, DatabaseResult};
use crate::model::folder::{Folder, FolderId};
use crate::model::user::UserId;
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_folders(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<Folder>> {
    const ID_IDX: usize = 8;

    let folders = query("SELECT UserId, ParentId, Name, Description, Emoji, Color, CreatedAt, UpdatedAt, ID FROM Folder WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(folders.into_iter().map(|row| {
        let id: FolderId = row.get(ID_IDX);
        folder(row, id)
    }).collect())
}

pub async fn get_folder_by_id(pool: &DatabasePool, folder_id: FolderId) -> DatabaseResult<Option<Folder>> {
    let res = query("SELECT UserId, ParentId,Name, Description, Emoji, Color, CreatedAt, UpdatedAt FROM Folder WHERE ID = ?")
        .bind(folder_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(res.map(|row| folder(row, folder_id)))
}

pub async fn get_folder_user_id(pool: &DatabasePool, folder_id: FolderId) -> DatabaseResult<UserId> {
    let user_id = query("SELECT UserId FROM Folder WHERE ID = ?")
        .bind(folder_id)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(user_id.get(0))
}

pub async fn create_folder(pool: &DatabasePool, user_id: UserId, folder: &Folder) -> DatabaseResult<FolderId> {
    let res = query("INSERT INTO Folder (UserId, ParentId, Name, Description, Emoji, Color) VALUES (?, ?, ?, ?, ?, ?) RETURNING ID")
        .bind(user_id)
        .bind(folder.parent_id)
        .bind(&folder.name)
        .bind(&folder.description)
        .bind(&folder.emoji)
        .bind(folder.color)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(res.get(0))
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

fn folder(row: MySqlRow, id: FolderId) -> Folder {
    let user_id = row.get(0);
    let parent_id = row.get(1);
    let name = row.get(2);
    let description = row.get(3);
    let emoji = row.get(4);
    let color = row.get(5);
    let created_at = row.get(6);
    let updated_at = row.get(7);

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