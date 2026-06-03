use crate::database::{DatabasePool, DatabaseResult};
use crate::model::auth::{AccountInfo, SessionResponse};
use crate::model::user::{UserId, UserInfo};
use crate::security::{hash, random_string, verify, SESSION_TOKEN_LENGTH};
use anyhow::anyhow;
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::{query, Arguments, Executor, Row, Statement};
use uuid::Uuid;

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

pub async fn login(pool: &DatabasePool, device_name: &str, user_name: &str, password: &str) -> DatabaseResult<Option<AccountInfo>> {
    let user = query("SELECT ID, Name, Email, AvatarUrl, Description, Color, System, CreatedAt, FriendCode, Password FROM User WHERE Name=?")
        .bind(user_name)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let user_id: UserId = user.get("ID");
        let password_hash: String = user.get("Password");

        if verify(&password_hash, password).await.is_ok() {
            let token = random_string(SESSION_TOKEN_LENGTH);

            let token_id = query("INSERT INTO Session (UserID, Token, Name) VALUES (?, ?, ?) RETURNING ID")
                .bind(user_id)
                .bind(&token)
                .bind(device_name)
                .fetch_one(pool.as_ref())
                .await?;
            let token_id = token_id.get(0);

            let created_at = user.get("CreatedAt");
            let friend_code: Uuid = user.get("FriendCode");
            let friend_code = friend_code.simple().to_string();
            let email = user.get("Email");
            let user = user_info(user, email);
            Ok(Some(AccountInfo {
                session: SessionResponse {
                    id: token_id,
                    token,
                },
                created_at,
                friend_code,
                user,
            }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
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
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub async fn change_password(pool: &DatabasePool, id: UserId, old_password: &str, new_password: &str) -> DatabaseResult<bool> {
    let user = query("SELECT Password FROM User WHERE ID=?")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(user) = user {
        let password_hash: String = user.get("Password");
        if verify(&password_hash, old_password).await.is_ok() {
            let password_hash = hash(new_password).await.map_err(|e| anyhow!("{:?}", e))?;

            query("UPDATE User SET Password = ? WHERE ID=?")
                .bind(password_hash)
                .bind(id)
                .execute(pool.as_ref())
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
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