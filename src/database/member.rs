use crate::database::{DatabaseExecutor, DatabasePool, DatabaseResult};
use crate::model::folder::FolderId;
use crate::model::member::{Member, MemberId};
use crate::model::user::UserId;
use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_member_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<MemberId>> {
    let ids = query("SELECT ID FROM Member WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_updated_members(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<Member>> {
    let updated = query("SELECT ID, UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom FROM Member WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    if updated.is_empty() {
        return Ok(vec![]);
    }

    let folders = {
        let placeholders = updated.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
        let sql = format!("SELECT MemberId, FolderId FROM MemberFolder WHERE MemberId IN ({placeholders}) AND UserId = ?");
        let mut query = query(sql.as_str());
        for member in &updated {
            let id: MemberId = member.get("ID");
            query = query.bind(id);
        }
        query
            .bind(user_id)
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(updated.into_iter().map(|row| {
        let id = row.get("ID");
        let folders = folders.iter()
            .filter(|f| f.get::<MemberId, _>("MemberId") == id)
            .map(|f| f.get("FolderId"))
            .collect();
        member(row, id, folders)
    }).collect())
}

pub async fn get_members(pool: &DatabasePool, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<Vec<Member>> {
    let members = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom, ID
FROM Member m
WHERE UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketMember pm
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pm.BucketId AND pf.UserId = pm.UserId
             WHERE pm.MemberId = m.ID AND pf.FriendId = ?
)
"#)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_all(pool.as_ref())
            .await?
    } else {
        query("SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom, ID FROM Member WHERE UserId = ?")
            .bind(user_id)
            .fetch_all(pool.as_ref())
            .await?
    };
    if members.is_empty() {
        return Ok(vec![]);
    }

    let folders = {
        let placeholders = members.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
        let sql = if friend_viewer.is_some() {
            format!(r#"
SELECT MemberId, FolderId
FROM MemberFolder mf
WHERE MemberId IN ({placeholders}) AND UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketFolder pf
             INNER JOIN PrivacyBucketFriend pfr
             ON pfr.BucketId = pf.BucketId AND pfr.UserId = pf.UserId
             WHERE pf.FolderId = mf.FolderId AND pfr.FriendId = ?
)
"#)
        } else {
            format!("SELECT MemberId, FolderId FROM MemberFolder WHERE MemberId IN ({placeholders}) AND UserId = ?")
        };
        let mut query = query(sql.as_str());
        for member in &members {
            let id: MemberId = member.get("ID");
            query = query.bind(id);
        }
        query = query.bind(user_id);
        if let Some(friend_viewer) = friend_viewer {
            query = query.bind(friend_viewer);
        }
        query.fetch_all(pool.as_ref())
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

pub async fn get_member_by_id(pool: &DatabasePool, member_id: MemberId, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<Option<Member>> {
    let res = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom
FROM Member m
WHERE ID = ? AND UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketMember pm
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pm.BucketId AND pf.UserId = pm.UserId
             WHERE pm.MemberId = m.ID AND pf.FriendId = ?
)
"#)
            .bind(member_id)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_optional(pool.as_ref())
            .await?
    } else {
        query("SELECT UserId, Name, Pronouns, AvatarUrl, Description, Color, CreatedAt, UpdatedAt, Archived, Custom FROM Member WHERE ID = ? AND UserId = ?")
            .bind(member_id)
            .bind(user_id)
            .fetch_optional(pool.as_ref())
            .await?
    };
    if res.is_none() {
        return Ok(None);
    }
    let res = res.unwrap();

    let folders = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT FolderId
FROM MemberFolder mf
WHERE MemberId = ? AND UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketFolder pf
             INNER JOIN PrivacyBucketFriend pfr
             ON pfr.BucketId = pf.BucketId AND pfr.UserId = pf.UserId
             WHERE pf.FolderId = mf.FolderId AND pfr.FriendId = ?
)
"#)
            .bind(member_id)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_all(pool.as_ref())
            .await?
    } else {
        query("SELECT FolderId FROM MemberFolder WHERE MemberId = ? AND UserId = ?")
            .bind(member_id)
            .bind(user_id)
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(Some(member(res, member_id, folders.into_iter().map(|row| row.get(0)).collect())))
}

pub async fn create_member<'a, E: DatabaseExecutor<'a>>(executor: E, member: &Member) -> DatabaseResult<MemberId> {
    let id = query("INSERT INTO Member (UserId, Name, Pronouns, AvatarUrl, Description, Color, Custom) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING ID")
        .bind(member.user_id)
        .bind(&member.name)
        .bind(&member.pronouns)
        .bind(&member.avatar)
        .bind(&member.description)
        .bind(member.color)
        .bind(member.custom)
        .fetch_one(executor)
        .await?;

    Ok(id.get(0))
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
        add_member_folder(&mut *tx, member_id, user_id, *folder_id).await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn add_member_folder<'a, E: DatabaseExecutor<'a>>(executor: E, member_id: MemberId, user_id: UserId, folder_id: FolderId) -> DatabaseResult<()> {
    query("INSERT INTO MemberFolder (UserId, MemberId, FolderId) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(member_id)
        .bind(folder_id)
        .execute(executor)
        .await?;
    
    Ok(())
}

pub async fn get_member_owner(pool: &DatabasePool, member_id: MemberId) -> DatabaseResult<Option<UserId>> {
    let owner = query("SELECT UserId FROM Member WHERE ID = ?")
        .bind(member_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(owner.map(|row| row.get("UserId")))
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