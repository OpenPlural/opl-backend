use crate::database::{to_web_error, DatabasePool, DatabaseResult};
use crate::model::auth::AccountInfo;
use crate::model::user::{UserId, UserInfo};
use crate::security::{hash, random_string, sha256, verify, SESSION_TOKEN_LENGTH};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::{query, Arguments, Executor, Row, Statement};
use uuid::Uuid;
use crate::error::WebError;

pub async fn register(pool: &DatabasePool, user_name: &str, password: &str, system: bool) -> DatabaseResult<bool> {
    let user = query("SELECT 1 FROM User WHERE Name=?")
        .bind(user_name)
        .fetch_optional(pool.as_ref())
        .await?;

    if user.is_some() {
        return Ok(false);
    }

    let password_hash = hash(password).await.map_err(|e| anyhow!("{:?}", e))?;

    query("INSERT INTO User (Name, Password, System) VALUES (?, ?, ?)")
        .bind(user_name)
        .bind(password_hash)
        .bind(system)
        .execute(pool.as_ref())
        .await?;

    Ok(true)
}

pub async fn login(pool: &DatabasePool, device_name: &str, user_name: &str, password: &str) -> Result<(AccountInfo, String), WebError> {
    let user = query("SELECT ID, Name, Email, AvatarUrl, Description, Color, System, CreatedAt, FriendCode, Password, AccountDisabled FROM User WHERE Name=?")
        .bind(user_name)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|err| to_web_error(anyhow!(err)))?;

    if let Some(user) = user {
        if user.get("AccountDisabled") {
            return Err(WebError::AccountDisabled);
        }

        let user_id: UserId = user.get("ID");
        let password_hash: String = user.get("Password");

        if verify(&password_hash, password).await.is_ok() {
            let token = random_string(SESSION_TOKEN_LENGTH);
            let token_hash = sha256(&token);

            let token_id = query("INSERT INTO Session (UserID, Token, Name) VALUES (?, ?, ?) RETURNING ID")
                .bind(user_id)
                .bind(token_hash)
                .bind(device_name)
                .fetch_one(pool.as_ref())
                .await
                .map_err(|err| to_web_error(anyhow!(err)))?;
            let token_id = token_id.get(0);

            let created_at = user.get("CreatedAt");
            let friend_code: Uuid = user.get("FriendCode");
            let friend_code = friend_code.simple().to_string();
            let email = user.get("Email");
            let user = user_info(user, email);
            return Ok((AccountInfo {
                session: Some(token_id),
                created_at,
                friend_code,
                user,
            }, token));
        } else {
            query("UPDATE User SET WrongPasswordEntries=WrongPasswordEntries+1, AccountDisabled=IF(WrongPasswordEntries >= 10, TRUE, AccountDisabled) WHERE ID=?")
                .bind(user_id)
                .execute(pool.as_ref())
                .await
                .map_err(|err| to_web_error(anyhow!(err)))?;
        }
    }
    Err(WebError::InvalidCredentials)
}

pub async fn delete(pool: &DatabasePool, id: UserId, password: &str) -> DatabaseResult<bool> {
    let user = query("SELECT Password FROM User WHERE ID=?")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let password_hash: String = user.get("Password");
        if verify(&password_hash, password).await.is_ok() {
            query("DELETE FROM User WHERE ID=?")
                .bind(id)
                .execute(pool.as_ref())
                .await?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn verify_and_change_password(pool: &DatabasePool, id: UserId, old_password: &str, new_password: &str) -> DatabaseResult<bool> {
    let user = query("SELECT Password FROM User WHERE ID=?")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let password_hash: String = user.get("Password");
        if verify(&password_hash, old_password).await.is_ok() {
            change_password(pool, id, new_password).await?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn change_password(pool: &DatabasePool, id: UserId, new_password: &str) -> DatabaseResult<()> {
    let password_hash = hash(new_password).await.map_err(|e| anyhow!("{:?}", e))?;

    query("UPDATE User SET Password = ? WHERE ID=?")
        .bind(password_hash)
        .bind(id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn reset_password(pool: &DatabasePool, name: &str, reset_token: &str, new_password: &str) -> DatabaseResult<bool> {
    let user = query("SELECT ID, PasswordResetToken FROM User WHERE Name=?")
        .bind(name)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let id: UserId = user.get("ID");
        let token: Option<String> = user.get("PasswordResetToken");
        if let Some(token) = token {
            let reset_token = sha256(reset_token);
            if reset_token == token {
                let password_hash = hash(new_password).await.map_err(|e| anyhow!("{:?}", e))?;

                query("UPDATE User SET Password = ?, PasswordResetToken = NULL, PasswordResetTokenExpires = NULL WHERE ID=?")
                    .bind(password_hash)
                    .bind(id)
                    .execute(pool.as_ref())
                    .await?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub async fn is_disabled(pool: &DatabasePool, id: UserId) -> DatabaseResult<Option<bool>> {
    let user = query("SELECT AccountDisabled FROM User WHERE ID=?")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(user.map(|row| row.get("AccountDisabled")))
}

pub async fn set_disabled(pool: &DatabasePool, id: UserId, disabled: bool) -> DatabaseResult<()> {
    query("UPDATE User SET AccountDisabled = ? WHERE ID=?")
        .bind(disabled)
        .bind(id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn can_reset_password(pool: &DatabasePool, id: UserId) -> DatabaseResult<Option<bool>> {
    let user = query("SELECT PasswordResetToken FROM User WHERE ID=?")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(user.map(|row| {
        let token: Option<String> = row.get("PasswordResetToken");
        token.is_some()
    }))
}

pub async fn update_user(pool: &DatabasePool, user: &UserInfo) -> DatabaseResult<()> {
    query("UPDATE User SET Name = ?, Email = ?, AvatarUrl = ?, Description = ?, Color = ?, System = ? WHERE ID=?")
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.avatar)
        .bind(&user.description)
        .bind(user.color)
        .bind(user.system)
        .bind(user.id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn resolve_friend_code(pool: &DatabasePool, friend_code: &Uuid) -> DatabaseResult<Option<UserId>> {
    let user_id = query("SELECT ID FROM User WHERE FriendCode=?")
        .bind(friend_code)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(user_id.map(|row| row.get(0)))
}

pub async fn get_user_by_id(pool: &DatabasePool, user_id: UserId, with_email: bool) -> DatabaseResult<Option<(UserInfo, String)>> {
    let user = query("SELECT ID, Name, Email, AvatarUrl, Description, Color, System, FriendCode FROM User WHERE ID=?")
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let friend_code: Uuid = user.get("FriendCode");
        let friend_code = friend_code.simple().to_string();
        let email = if with_email {
            user.get("Email")
        } else {
            None
        };
        let user = user_info(user, email);
        Ok(Some((user, friend_code)))
    } else {
        Ok(None)
    }
}

pub async fn get_user_creation_date(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Option<DateTime<Utc>>> {
    let date = query("SELECT CreatedAt FROM User WHERE ID=?")
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(date) = date {
        Ok(Some(date.get("CreatedAt")))
    } else {
        Ok(None)
    }
}

pub async fn get_username(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Option<String>> {
    let username = query("SELECT Name FROM User WHERE ID=?")
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(username.map(|row| row.get("Name")))
}

pub async fn get_users_by_ids(pool: &DatabasePool, user_ids: &[UserId]) -> DatabaseResult<Vec<UserInfo>> {
    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = user_ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
    let sql = format!("SELECT ID, Name, AvatarUrl, Description, Color, System FROM User WHERE ID IN ({placeholders})");

    let mut args = MySqlArguments::default();
    for user_id in user_ids {
        args.add(*user_id).map_err(|e| anyhow!("{:?}", e))?;
    }
    let statement = pool.prepare(&sql).await?;
    let users = statement.query_with(args).fetch_all(pool.as_ref()).await?;

    Ok(users.into_iter().map(|user| user_info(user, None)).collect())
}

pub async fn change_friend_code(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<()> {
    query("UPDATE User SET FriendCode = RANDOM_BYTES(16) WHERE ID=?")
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_friend_code(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Option<String>> {
    let row = query("SELECT FriendCode FROM User WHERE ID=?")
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;
    
    Ok(row.map(|row| {
        let friend_code: Uuid = row.get("FriendCode");
        friend_code.simple().to_string()
    }))
}

pub async fn clear_expired_password_reset_tokens(pool: &DatabasePool) -> DatabaseResult<()> {
    query("UPDATE User SET PasswordResetToken = NULL, PasswordResetTokenExpires = NULL WHERE PasswordResetToken IS NOT NULL AND PasswordResetTokenExpires IS NOT NULL AND PasswordResetTokenExpires < CURRENT_TIMESTAMP()")
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

fn user_info(row: MySqlRow, email: Option<String>) -> UserInfo {
    let user_id = row.get("ID");
    let user_name = row.get("Name");
    let avatar_url = row.get("AvatarUrl");
    let description = row.get("Description");
    let color = row.get("Color");
    let system = row.get("System");

    UserInfo {
        id: user_id,
        name: user_name,
        email,
        avatar: avatar_url,
        description,
        color,
        system,
    }
}