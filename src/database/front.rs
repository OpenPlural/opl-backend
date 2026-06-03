use std::collections::HashMap;
use sqlx::Arguments;
use anyhow::anyhow;
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::front::{FrontEntry, FrontEntryId};
use crate::model::user::{UserId, UserInfo};
use chrono::{DateTime, Utc};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::{query, Executor, Row, Statement};
use crate::model::friend::Friend;
use crate::model::member::MemberId;

pub async fn fill_front_text(pool: &DatabasePool, viewer: UserId, users: Vec<UserInfo>) -> DatabaseResult<Vec<Friend>> {
    if users.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = users.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
    let sql = format!(r#"
SELECT f.UserId, m.Name FROM Front f JOIN Member m ON m.ID = f.ID WHERE f.UserId IN ({placeholders}) AND f.EndedAt IS NULL AND EXISTS (
    SELECT 1 FROM PrivacyBucketMember pm
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pm.BucketId AND pf.UserId = pm.UserId
             WHERE pm.MemberId = f.MemberId AND pf.FriendId = ?
)
"#);

    let mut args = MySqlArguments::default();
    for user in &users {
        args.add(user.id).map_err(|e| anyhow!("{:?}", e))?;
    }
    let statement = pool.prepare(&sql).await?;
    let front = statement.query_with(args).bind(viewer).fetch_all(pool.as_ref()).await?;

    let mut map: HashMap<UserId, Vec<String>> = HashMap::with_capacity(users.len());
    for row in front {
        let user_id: UserId = row.get("UserId");
        let name: String = row.get("Name");

        if let Some(list) = map.get_mut(&user_id) {
            list.push(name);
        } else {
            let list = vec![name];
            map.insert(user_id, list);
        }
    }
    Ok(users.into_iter().map(|user| Friend {
        front_text: map.get(&user.id).map(|list| list.join(", ")),
        user,
    }).collect())
}

pub async fn get_front_history(pool: &DatabasePool, user_id: UserId, page: u32) -> DatabaseResult<Vec<FrontEntry>> {
    let entries = query("SELECT ID, MemberId, StartedAt, EndedAt, Comment, UpdatedAt FROM Front WHERE UserId = ? ORDER BY StartedAt DESC LIMIT ?, 50")
        .bind(user_id)
        .bind(page * 50)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(entries.into_iter().map(|row| {
        let ended_at = row.get("EndedAt");
        front_entry(row, user_id, ended_at)
    }).collect())
}

pub async fn get_front_history_of_member(pool: &DatabasePool, user_id: UserId, member_id: MemberId, page: u32) -> DatabaseResult<Vec<FrontEntry>> {
    let entries = query("SELECT ID, MemberId, StartedAt, EndedAt, Comment, UpdatedAt FROM Front WHERE UserId = ? AND MemberId = ? ORDER BY StartedAt DESC LIMIT ?, 50")
        .bind(user_id)
        .bind(member_id)
        .bind(page * 50)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(entries.into_iter().map(|row| {
        let ended_at = row.get("EndedAt");
        front_entry(row, user_id, ended_at)
    }).collect())
}

pub async fn get_current_front_entries(pool: &DatabasePool, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<Vec<FrontEntry>> {
    let entries = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT ID, MemberId, StartedAt, Comment, UpdatedAt
FROM Front f
WHERE UserId = ? AND EndedAt IS NULL AND EXISTS (
    SELECT 1 FROM PrivacyBucketMember pm
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pm.BucketId AND pf.UserId = pm.UserId
             WHERE pm.MemberId = f.MemberId AND pf.FriendId = ?
)
"#)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_all(pool.as_ref())
            .await?
    } else {
        query("SELECT ID, MemberId, StartedAt, Comment, UpdatedAt FROM Front WHERE UserId = ? AND EndedAt IS NULL")
            .bind(user_id)
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(entries.into_iter().map(|row| front_entry(row, user_id, None)).collect())
}

pub async fn get_front_entry_by_id(pool: &DatabasePool, entry_id: FrontEntryId, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<Option<FrontEntry>> {
    let entry = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT ID, MemberId, StartedAt, EndedAt, Comment, UpdatedAt
FROM Front f
WHERE ID = ? AND UserId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketMember pm
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pm.BucketId AND pf.UserId = pm.UserId
             WHERE pm.MemberId = f.MemberId AND pf.FriendId = ?
)
"#)
            .bind(entry_id)
            .bind(user_id)
            .bind(friend_viewer)
            .fetch_optional(pool.as_ref())
            .await?
    } else {
        query("SELECT ID, MemberId, StartedAt, EndedAt, Comment, UpdatedAt FROM Front WHERE ID = ? AND UserId = ?")
            .bind(entry_id)
            .bind(user_id)
            .fetch_optional(pool.as_ref())
            .await?
    };
    Ok(entry.map(|row| {
        let ended_at = row.get("EndedAt");
        front_entry(row, user_id, ended_at)
    }))
}

pub async fn get_active_front_entry_by_member(pool: &DatabasePool, user_id: UserId, member_id: MemberId) -> DatabaseResult<Option<FrontEntry>> {
    let entry = query("SELECT ID, MemberId, StartedAt, Comment, UpdatedAt FROM Front WHERE UserId = ? AND MemberId = ? AND EndedAt IS NULL")
        .bind(user_id)
        .bind(member_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(entry.map(|row| front_entry(row, user_id, None)))
}

pub async fn is_fronting(pool: &DatabasePool, user_id: UserId, member_id: MemberId) -> DatabaseResult<bool> {
    let res = query("SELECT 1 FROM Front WHERE UserId = ? AND MemberId = ? AND EndedAt IS NULL")
        .bind(user_id)
        .bind(member_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(res.is_some())
}

pub async fn add_front_entry(pool: &DatabasePool, entry: &FrontEntry) -> DatabaseResult<FrontEntryId> {
    let id = query("INSERT INTO Front (UserId, MemberId, StartedAt, EndedAt, Comment) VALUES (?, ?, ?, ?, ?) RETURNING ID")
        .bind(entry.user_id)
        .bind(entry.member_id)
        .bind(entry.started_at)
        .bind(entry.ended_at)
        .bind(entry.comment.clone())
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn delete_front_entry(pool: &DatabasePool, entry_id: FrontEntryId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM Front WHERE ID = ? AND UserId = ?")
        .bind(entry_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_front_entry(pool: &DatabasePool, entry: &FrontEntry) -> DatabaseResult<()> {
    query("UPDATE Front SET MemberId = ?, StartedAt = ?, EndedAt = ?, Comment = ? WHERE ID = ? AND UserId = ?")
        .bind(entry.member_id)
        .bind(entry.started_at)
        .bind(entry.ended_at)
        .bind(entry.comment.clone())
        .bind(entry.id)
        .bind(entry.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

fn front_entry(row: MySqlRow, user_id: UserId, ended_at: Option<DateTime<Utc>>) -> FrontEntry {
    let started_at: DateTime<Utc> = row.get("StartedAt");
    FrontEntry {
        id: row.get("ID"),
        user_id,
        member_id: row.get("MemberId"),
        started_at,
        ended_at,
        comment: row.get("Comment"),
        updated_at: row.get("UpdatedAt"),
    }
}
