use crate::database::{DatabasePool, DatabaseResult};
use crate::model::front::{FrontEntry, FrontEntryId};
use crate::model::member::MemberId;
use crate::model::user::UserId;
use chrono::{DateTime, Utc};
use sqlx::{query, Row};

pub async fn get_current_front_member_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<MemberId>> {
    let entries = query("SELECT MemberId FROM Front WHERE UserId = ? AND EndedAt IS NULL")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(entries.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_current_front_entries(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FrontEntry>> {
    let entries = query("SELECT ID, MemberId, StartedAt, Comment FROM Front WHERE UserId = ? AND EndedAt IS NULL")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(entries.into_iter().map(|row| {
        let started_at: DateTime<Utc> = row.get(2);
        FrontEntry {
            id: row.get(0),
            member: row.get(1),
            started_at: started_at.to_rfc3339(),
            ended_at: None,
            comment: row.get(3),
        }
    }).collect())
}

pub async fn get_current_front_entry(pool: &DatabasePool, member_id: MemberId) -> DatabaseResult<Option<FrontEntry>> {
    let entry = query("SELECT ID, StartedAt, Comment FROM Front WHERE MemberId = ? AND EndedAt IS NULL")
        .bind(member_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(entry) = entry {
        let started_at: DateTime<Utc> = entry.get(1);
        Ok(Some(FrontEntry {
            id: entry.get(0),
            member: member_id,
            started_at: started_at.to_rfc3339(),
            ended_at: None,
            comment: entry.get(2),
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_front_user_id(pool: &DatabasePool, entry_id: FrontEntryId) -> DatabaseResult<UserId> {
    let user_id = query("SELECT UserId FROM Front WHERE ID = ?")
        .bind(entry_id)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(user_id.get(0))
}

pub async fn check_fronting(pool: &DatabasePool, member_id: MemberId) -> DatabaseResult<bool> {
    let entry = query("SELECT 1 FROM Front WHERE MemberId = ? AND EndedAt IS NULL")
        .bind(member_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(entry.is_some())
}

pub async fn add_front_entry(pool: &DatabasePool, user_id: UserId, member_id: MemberId) -> DatabaseResult<FrontEntryId> {
    let res = query("INSERT INTO Front (UserId, MemberId) VALUES (?, ?) RETURNING ID")
        .bind(user_id)
        .bind(member_id)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(res.get(0))
}

pub async fn add_front_entry_full(pool: &DatabasePool, user_id: UserId, member_id: MemberId, started_at: &DateTime<Utc>, ended_at: Option<&DateTime<Utc>>, comment: &Option<String>) -> DatabaseResult<FrontEntryId> {
    let res = query("INSERT INTO Front (UserId, MemberId, StartedAt, EndedAt, Comment) VALUES (?, ?, ?, ?, ?) RETURNING ID")
        .bind(user_id)
        .bind(member_id)
        .bind(started_at)
        .bind(ended_at)
        .bind(comment)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(res.get(0))
}

pub async fn edit_front_comment(pool: &DatabasePool, entry_id: FrontEntryId, comment: &Option<String>) -> DatabaseResult<()> {
    query("UPDATE Front SET Comment = ? WHERE ID = ?")
        .bind(comment)
        .bind(entry_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_start_time(pool: &DatabasePool, entry_id: FrontEntryId, started_at: &DateTime<Utc>) -> DatabaseResult<()> {
    query("UPDATE Front SET StartedAt = ? WHERE ID = ?")
        .bind(started_at)
        .bind(entry_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_end_time(pool: &DatabasePool, entry_id: FrontEntryId, ended_at: &DateTime<Utc>) -> DatabaseResult<()> {
    query("UPDATE Front SET EndedAt = ? WHERE ID = ?")
        .bind(ended_at)
        .bind(entry_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn end_current_front(pool: &DatabasePool, member_id: MemberId, ended_at: &DateTime<Utc>) -> DatabaseResult<()> {
    query("UPDATE Front SET EndedAt = ? WHERE MemberId = ? AND EndedAt IS NULL")
        .bind(ended_at)
        .bind(member_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}
