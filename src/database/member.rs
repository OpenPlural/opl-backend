use crate::database::{DatabasePool, DatabaseResult};
use crate::model::folder::FolderId;
use crate::model::member::{Member, MemberId};
use crate::model::user::UserId;
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_members(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<Member>> {
    const ID_IDX: usize = 10;

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
            let id: MemberId = member.get(ID_IDX);
            query = query.bind(id);
        }
        query
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(members.into_iter().map(|row| {
        let id = row.get(ID_IDX);
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

pub async fn get_member_by_id(pool: &DatabasePool, member_id: MemberId) -> DatabaseResult<Option<Member>> {
    let res = query("SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom FROM Member WHERE ID = ?")
        .bind(member_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if res.is_none() {
        return Ok(None);
    }
    let res = res.unwrap();

    let folders = query("SELECT FolderId FROM MemberFolder WHERE MemberId = ?")
        .bind(member_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(Some(member(res, member_id, folders.into_iter().map(|row| row.get(0)).collect())))
}

pub async fn get_member_user_id(pool: &DatabasePool, member_id: MemberId) -> DatabaseResult<UserId> {
    let user_id = query("SELECT UserId FROM Member WHERE ID = ?")
        .bind(member_id)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(user_id.get(0))
}

pub async fn create_member(pool: &DatabasePool, user_id: UserId, member: &Member) -> DatabaseResult<MemberId> {
    let res = query("INSERT INTO Member (UserId, Name, Pronouns, AvatarUrl, Description, Color, Custom) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING ID")
        .bind(user_id)
        .bind(&member.name)
        .bind(&member.pronouns)
        .bind(&member.avatar)
        .bind(&member.description)
        .bind(member.color)
        .bind(member.custom)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(res.get(0))
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

pub async fn edit_member_folders(pool: &DatabasePool, member_id: MemberId, folder_ids: &[FolderId]) -> DatabaseResult<()> {
    let mut tx = pool.begin().await?;

    query("DELETE FROM MemberFolder WHERE MemberId = ?")
        .bind(member_id)
        .execute(&mut *tx)
        .await?;

    for folder_id in folder_ids {
        query("INSERT INTO MemberFolder (MemberId, FolderId) VALUES (?, ?)")
            .bind(member_id)
            .bind(folder_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

fn member(row: MySqlRow, id: MemberId, folders: Vec<FolderId>) -> Member {
    let user_id = row.get(0);
    let name = row.get(1);
    let pronouns = row.get(2);
    let avatar = row.get(3);
    let description = row.get(4);
    let color = row.get(5);
    let created_at = row.get(6);
    let updated_at = row.get(7);
    let archived = row.get(8);
    let custom = row.get(9);

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