use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::middleware::RequestToken;
use crate::model::apikey::{ApiKey, ApiKeyId};
use crate::model::user::UserId;

pub async fn check_api_key(pool: &DatabasePool, api_key: &str) -> DatabaseResult<Option<RequestToken>> {
    let token = query("SELECT UserId, WritePermission FROM ApiKey WHERE Token = ?")
        .bind(api_key)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(token) = token {
        let user_id = token.get("UserId");
        let write = token.get("WritePermission");
        Ok(Some(RequestToken {
            session_id: None,
            user_id,
            write,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_api_keys(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<ApiKey>> {
    let tokens = query("SELECT ID, Name, WritePermission, CreatedAt FROM ApiKey WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(tokens.into_iter().map(|row| ApiKey {
        id: row.get("ID"),
        user_id,
        token: None,
        name: row.get("Name"),
        write: row.get("WritePermission"),
        created_at: row.get("CreatedAt")
    }).collect())
}

pub async fn create_api_key(pool: &DatabasePool, api_key: &ApiKey, token: &str) -> DatabaseResult<ApiKeyId> {
    let id = query("INSERT INTO ApiKey (UserId, Token, Name, WritePermission) VALUES (?, ?, ?, ?) RETURNING ID")
        .bind(api_key.user_id)
        .bind(token)
        .bind(&api_key.name)
        .bind(api_key.write)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn delete_api_key(pool: &DatabasePool, token_id: ApiKeyId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM ApiKey WHERE ID = ? AND UserId = ?")
        .bind(token_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}