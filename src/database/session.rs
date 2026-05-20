use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::middleware::RequestToken;
use crate::model::session::{SessionToken, TokenId};
use crate::model::user::UserId;

pub async fn check_session(pool: &DatabasePool, session_id: &str) -> DatabaseResult<Option<RequestToken>> {
    let session = query("SELECT ID, UserId FROM Session WHERE Token = ? AND ExpiresAt > NOW()")
        .bind(session_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(session) = session {
        let token_id: TokenId = session.get("ID");
        let user_id: UserId = session.get("UserId");

        Ok(Some(RequestToken {
            token_id,
            user_id,
            write: true,
            admin: true,
        }))
    } else {
        Ok(None)
    }
}

pub async fn extend_session(pool: &DatabasePool, token_id: TokenId) -> DatabaseResult<()> {
    query("UPDATE Session SET ExpiresAt = DATE_ADD(NOW(), INTERVAL 7 DAY) WHERE ID = ?")
        .bind(token_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_sessions(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<SessionToken>> {
    let sessions = query("SELECT ID, Name, CreatedAt, ExpiresAt FROM Session WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(sessions.into_iter().map(|row| SessionToken {
        id: row.get("ID"),
        name: row.get("Name"),
        created_at: row.get("CreatedAt"),
        expires_at: row.get("ExpiresAt")
    }).collect())
}

pub async fn delete_session(pool: &DatabasePool, user_id: UserId, token_id: TokenId) -> DatabaseResult<()> {
    query("DELETE FROM Session WHERE ID = ? AND UserId = ?")
        .bind(token_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}