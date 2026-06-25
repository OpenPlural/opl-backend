use sqlx::{query, Row};
use sqlx::mysql::MySqlRow;
use uuid::Uuid;
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::friend::{FriendRequest, FriendSettings, PERMISSION_LEVEL_NOTIFICATIONS};
use crate::model::user::UserId;

pub async fn get_friend_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<UserId>> {
    let friends = query("SELECT FriendId FROM Friend WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friends.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_notified_friend_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<(UserId, bool)>> {
    let friends = query("SELECT s.UserId, ns.ReplaceFrontChange FROM Friend f JOIN Friend s ON s.UserId = f.FriendId LEFT JOIN NotificationSettings ns ON ns.UserId = f.FriendId WHERE f.UserId = ? AND f.PermissionLevel >= ? AND s.NotifyMe")
        .bind(user_id)
        .bind(PERMISSION_LEVEL_NOTIFICATIONS)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friends.into_iter().map(|row| {
        let user_id: UserId = row.get(0);
        let replace_front_change: Option<bool> = row.get(1);
        (user_id, replace_front_change.unwrap_or(false))
    }).collect())
}

pub async fn get_incoming_friend_requests(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FriendRequest>> {
    let friend_requests = query("SELECT u.FriendCode, u.Name, u.System FROM FriendRequest fr JOIN User u ON u.ID = fr.FromUser WHERE fr.ToUser = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friend_requests.into_iter().map(friend_request).collect())
}

pub async fn get_outgoing_friend_requests(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FriendRequest>> {
    let friend_requests = query("SELECT u.FriendCode, u.Name, u.System FROM FriendRequest fr JOIN User u ON u.ID = fr.ToUser WHERE fr.FromUser = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friend_requests.into_iter().map(friend_request).collect())
}

pub async fn check_friendship(pool: &DatabasePool, user1: UserId, user2: UserId) -> DatabaseResult<bool> {
    let friendship = query("SELECT 1 FROM Friend WHERE UserId = ? AND FriendId = ?")
        .bind(user1)
        .bind(user2)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(friendship.is_some())
}

pub async fn check_friend_request(pool: &DatabasePool, from_user: UserId, to_user: UserId) -> DatabaseResult<bool> {
    let friend_request = query("SELECT 1 FROM FriendRequest WHERE FromUser = ? AND ToUser = ?")
        .bind(from_user)
        .bind(to_user)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(friend_request.is_some())
}

pub async fn send_friend_request(pool: &DatabasePool, from_user: UserId, to_user: UserId) -> DatabaseResult<()> {
    query("INSERT INTO FriendRequest (FromUser, ToUser) VALUES (?, ?)")
        .bind(from_user)
        .bind(to_user)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn accept_friend_request(pool: &DatabasePool, from_user: UserId, to_user: UserId) -> DatabaseResult<()> {
    let mut tx = pool.begin().await?;

    query("DELETE FROM FriendRequest WHERE FromUser = ? AND ToUser = ?")
        .bind(from_user)
        .bind(to_user)
        .execute(&mut *tx)
        .await?;

    query("INSERT INTO Friend (UserId, FriendId) VALUES (?, ?), (?, ?)")
        .bind(from_user)
        .bind(to_user)
        .bind(to_user)
        .bind(from_user)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn remove_friend_request(pool: &DatabasePool, from_user: UserId, to_user: UserId) -> DatabaseResult<()> {
    query("DELETE FROM FriendRequest WHERE FromUser = ? AND ToUser = ?")
        .bind(from_user)
        .bind(to_user)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn remove_friend(pool: &DatabasePool, user1: UserId, user2: UserId) -> DatabaseResult<()> {
    query("DELETE FROM Friend WHERE (UserId = ? AND FriendId = ?) OR (UserId = ? AND FriendId = ?)")
        .bind(user1)
        .bind(user2)
        .bind(user2)
        .bind(user1)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_friend_settings(pool: &DatabasePool, user_id: UserId, friend_id: UserId) -> DatabaseResult<Option<FriendSettings>> {
    let settings = query("SELECT PermissionLevel, NotifyMe FROM Friend WHERE UserId = ? AND FriendId = ?")
        .bind(user_id)
        .bind(friend_id)
        .fetch_optional(pool.as_ref())
        .await?;

    
    Ok(settings.map(|row| {
        let permission_level: i8 = row.get("PermissionLevel");
        let notify_me: bool = row.get("NotifyMe");

        FriendSettings {
            permission_level,
            notify_me,
        }
    }))
}

pub async fn update_friend_settings(pool: &DatabasePool, user_id: UserId, friend_id: UserId, settings: FriendSettings) -> DatabaseResult<()> {
    query("UPDATE Friend SET PermissionLevel = ?, NotifyMe = ? WHERE UserId = ? AND FriendId = ?")
        .bind(settings.permission_level)
        .bind(settings.notify_me)
        .bind(user_id)
        .bind(friend_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

fn friend_request(row: MySqlRow) -> FriendRequest {
    let code: Uuid = row.get("FriendCode");
    let code = code.simple().to_string();
    let name = row.get("Name");
    let system = row.get("System");

    FriendRequest {
        code,
        name,
        system,
    }
}