use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::middleware::{RequestToken, TokenId};
use crate::model::user::UserId;

pub async fn check_session(pool: &DatabasePool, session_id: &str) -> DatabaseResult<Option<RequestToken>> {
    let session = query("SELECT ID, UserId FROM Session WHERE Token = ? AND ExpiresAt > NOW()")
        .bind(session_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(session) = session {
        let token_id: TokenId = session.get(0);
        let user_id: UserId = session.get(1);

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