use crate::database::{DatabasePool, DatabaseResult};
use crate::model::front::{FrontEntry, FrontEntryId};
use crate::model::user::UserId;
use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_current_front_entries(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FrontEntry>> {
    let entries = query("SELECT ID, MemberId, StartedAt, Comment, UpdatedAt FROM Front WHERE UserId = ? AND EndedAt IS NULL")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(entries.into_iter().map(|row| front_entry(row, user_id)).collect())
}

pub async fn get_front_entry_by_id(pool: &DatabasePool, entry_id: FrontEntryId, user_id: UserId) -> DatabaseResult<Option<FrontEntry>> {
    let row = query("SELECT ID, MemberId, StartedAt, Comment, UpdatedAt FROM Front WHERE ID = ? AND UserId = ?")
        .bind(entry_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(row.map(|row| front_entry(row, user_id)))
}

pub async fn add_front_entry(pool: &DatabasePool, entry: &FrontEntry, started_at: DateTime<Utc>, ended_at: Option<DateTime<Utc>>) -> DatabaseResult<()> {
    query("INSERT INTO Front (ID, UserId, MemberId, StartedAt, EndedAt, Comment) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(entry.id)
        .bind(entry.user)
        .bind(entry.member)
        .bind(started_at)
        .bind(ended_at)
        .bind(entry.comment.clone())
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn delete_front_entry(pool: &DatabasePool, entry_id: FrontEntryId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM Front WHERE ID = ? AND UserId = ?")
        .bind(entry_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_front_entry(pool: &DatabasePool, entry: &FrontEntry, started_at: DateTime<Utc>, ended_at: Option<DateTime<Utc>>) -> DatabaseResult<()> {
    query("UPDATE Front SET MemberId = ?, StartedAt = ?, EndedAt = ?, Comment = ? WHERE ID = ? AND UserId = ?")
        .bind(entry.member)
        .bind(started_at)
        .bind(ended_at)
        .bind(entry.comment.clone())
        .bind(entry.id)
        .bind(entry.user)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

fn front_entry(row: MySqlRow, user_id: UserId) -> FrontEntry {
    let started_at: DateTime<Utc> = row.get("StartedAt");
    FrontEntry {
        id: row.get("ID"),
        user: user_id,
        member: row.get("MemberId"),
        started_at: started_at.to_rfc3339(),
        ended_at: None,
        comment: row.get("Comment"),
        updated_at: row.get("UpdatedAt"),
    }
}
