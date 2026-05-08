use crate::database::{DatabasePool, DatabaseResult};
use crate::model::folder::FolderId;
use crate::model::member::{Member, MemberId};
use crate::model::user::UserId;
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_members(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<Member>> {
    let members = query("SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom, ID FROM Member WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    if members.is_empty() {
        return Ok(vec![]);
    }

    let folders = {
        let placeholders = members.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
        let sql = format!("SELECT MemberId, FolderId FROM MemberFolder WHERE MemberId IN ({placeholders})");
        let mut query = query(sql.as_str());
        for member in &members {
            let id: MemberId = member.get("ID");
            query = query.bind(id);
        }
        query
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(members.into_iter().map(|row| {
        let id = row.get("ID");
        let folders = folders.iter()
            .filter(|row| {
                let member_id: MemberId = row.get(0);
                member_id == id
            })
            .map(|row| row.get(1))
            .collect();
        member(row, id, folders)
    }).collect())
}

pub async fn get_member_by_id(pool: &DatabasePool, member_id: MemberId, user_id: UserId) -> DatabaseResult<Option<Member>> {
    let res = query("SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom FROM Member WHERE ID = ? AND UserId = ?")
        .bind(member_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if res.is_none() {
        return Ok(None);
    }
    let res = res.unwrap();

    let folders = query("SELECT FolderId FROM MemberFolder WHERE MemberId = ? AND UserId = ?")
        .bind(member_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(Some(member(res, member_id, folders.into_iter().map(|row| row.get(0)).collect())))
}

pub async fn create_member(pool: &DatabasePool, user_id: UserId, member: &Member) -> DatabaseResult<()> {
    query("INSERT INTO Member (ID, UserId, Name, Pronouns, AvatarUrl, Description, Color, Custom) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(member.id)
        .bind(user_id)
        .bind(&member.name)
        .bind(&member.pronouns)
        .bind(&member.avatar)
        .bind(&member.description)
        .bind(member.color)
        .bind(member.custom)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn delete_member(pool: &DatabasePool, member_id: MemberId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM Member WHERE ID = ? AND UserId = ?")
        .bind(member_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_member(pool: &DatabasePool, member: &Member) -> DatabaseResult<()> {
    query("UPDATE Member SET Name = ?, Pronouns = ?, AvatarUrl = ?, Description = ?, Color = ? WHERE ID = ? AND UserId = ?")
        .bind(&member.name)
        .bind(&member.pronouns)
        .bind(&member.avatar)
        .bind(&member.description)
        .bind(member.color)
        .bind(member.id)
        .bind(member.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_member_folders(pool: &DatabasePool, member_id: MemberId, user_id: UserId, folder_ids: &[FolderId]) -> DatabaseResult<()> {
    let mut tx = pool.begin().await?;

    query("DELETE FROM MemberFolder WHERE MemberId = ? AND UserId = ?")
        .bind(member_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    for folder_id in folder_ids {
        query("INSERT INTO MemberFolder (UserId, MemberId, FolderId) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(member_id)
            .bind(folder_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

fn member(row: MySqlRow, id: MemberId, folders: Vec<FolderId>) -> Member {
    let user_id = row.get("UserId");
    let name = row.get("Name");
    let pronouns = row.get("Pronouns");
    let avatar = row.get("AvatarUrl");
    let description = row.get("Description");
    let color = row.get("Color");
    let created_at = row.get("CreatedAt");
    let updated_at = row.get("UpdatedAt");
    let archived = row.get("Archived");
    let custom = row.get("Custom");

    Member {
        id,
        user_id,
        name,
        pronouns,
        avatar,
        description,
        color,
        archived,
        custom,
        created_at,
        updated_at,
        folders,
    }
}