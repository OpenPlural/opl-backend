use crate::database::{DatabasePool, DatabaseResult};
use crate::security::{get_password_hash_algorithm, random_string, SESSION_TOKEN_LENGTH};
use anyhow::anyhow;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};
use sqlx::{query, Arguments, Executor, Row, Statement};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use uuid::Uuid;
use crate::model::auth::{SessionResponse, UserResponse};
use crate::model::user::UserId;

pub async fn register(pool: &DatabasePool, user_name: &str, password: &str, system: bool) -> DatabaseResult<bool> {
    let user = query("SELECT 1 FROM User WHERE Name=?")
        .bind(user_name)
        .fetch_optional(pool.as_ref())
        .await?;

    if user.is_some() {
        return Ok(false);
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = get_password_hash_algorithm().await
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("{:?}", e))?;
    let password_hash = password_hash.to_string();

    query("INSERT INTO User (Name, Password, System) VALUES (?, ?, ?)")
        .bind(user_name)
        .bind(password_hash)
        .bind(system)
        .execute(pool.as_ref())
        .await?;

    Ok(true)
}

pub async fn login(pool: &DatabasePool, device_name: &str, user_name: &str, password: &str) -> DatabaseResult<Option<UserResponse>> {
    let user = query("SELECT ID, Name, AvatarUrl, Description, Color, System, FriendCode, Password FROM User WHERE Name=?")
        .bind(user_name)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let user_id: UserId = user.get("ID");
        let password_hash: String = user.get("Password");
        let password_hash = PasswordHash::new(&password_hash).map_err(|e| anyhow!("{:?}", e))?;

        if get_password_hash_algorithm().await
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok() {
            let token = random_string(SESSION_TOKEN_LENGTH);

            query("INSERT INTO Session (UserID, Token, Name) VALUES (?, ?, ?)")
                .bind(user_id)
                .bind(&token)
                .bind(device_name)
                .execute(pool.as_ref())
                .await?;

            Ok(Some(user_response(user, true, Some(token))))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub async fn resolve_friend_code(pool: &DatabasePool, friend_code: &str) -> DatabaseResult<Option<UserId>> {
    let user_id = query("SELECT ID FROM User WHERE FriendCode=?")
        .bind(friend_code)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(user_id.map(|row| row.get(0)))
}

pub async fn get_user_by_id(pool: &DatabasePool, user_id: UserId, with_friend_code: bool) -> DatabaseResult<Option<UserResponse>> {
    let user = query("SELECT ID, Name, AvatarUrl, Description, Color, System, FriendCode FROM User WHERE ID=?")
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        Ok(Some(user_response(user, with_friend_code, None)))
    } else {
        Ok(None)
    }
}

pub async fn get_users_by_ids(pool: &DatabasePool, user_ids: &[UserId]) -> DatabaseResult<Vec<UserResponse>> {
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

    Ok(users.into_iter().map(|row| user_response(row, false, None)).collect())
}

fn user_response(row: MySqlRow, with_friend_code: bool, token: Option<String>) -> UserResponse {
    let user_id = row.get("ID");
    let user_name = row.get("Name");
    let avatar_url = row.get("AvatarUrl");
    let description = row.get("Description");
    let color = row.get("Color");
    let system = row.get("System");
    let friend_code = if with_friend_code {
        let uuid: Uuid = row.get("FriendCode");
        Some(uuid.simple().to_string())
    } else {
        None
    };

    UserResponse {
        session: token.map(|token| SessionResponse { token }),
        id: user_id,
        name: user_name,
        avatar: avatar_url,
        description,
        color,
        system,
        friend_code,
        folders: None,
        members: None,
        front: None,
    }
}