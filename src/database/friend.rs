use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::friend::{FriendRequest, FriendSettings};
use crate::model::user::UserId;

pub async fn get_friend_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<UserId>> {
    let friends = query("SELECT Friend1 FROM Friend WHERE Friend2 = ? UNION SELECT Friend2 FROM Friend WHERE Friend1 = ?")
        .bind(user_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friends.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_incoming_friend_requests(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FriendRequest>> {
    let friend_requests = query("SELECT u.FriendCode, u.Name FROM FriendRequest fr JOIN User u ON u.ID = fr.FromUser WHERE fr.ToUser = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friend_requests.into_iter().map(|row| FriendRequest {
        code: row.get("FriendCode"),
        name: row.get("Name"),
    }).collect())
}

pub async fn get_outgoing_friend_requests(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<FriendRequest>> {
    let friend_requests = query("SELECT u.FriendCode, u.Name FROM FriendRequest fr JOIN User u ON u.ID = fr.ToUser WHERE fr.FromUser = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(friend_requests.into_iter().map(|row| FriendRequest {
        code: row.get("FriendCode"),
        name: row.get("Name"),
    }).collect())
}

pub async fn check_friendship(pool: &DatabasePool, user1: UserId, user2: UserId) -> DatabaseResult<bool> {
    let friendship = query("SELECT 1 FROM Friend WHERE (Friend1 = ? AND Friend2 = ?) OR (Friend1 = ? AND Friend2 = ?)")
        .bind(user1)
        .bind(user2)
        .bind(user2)
        .bind(user1)
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

    query("INSERT INTO Friend (Friend1, Friend2) VALUES (?, ?)")
        .bind(from_user)
        .bind(to_user)
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
    let mut tx = pool.begin().await?;

    query("DELETE FROM Friend WHERE (Friend1 = ? AND Friend2 = ?) OR (Friend1 = ? AND Friend2 = ?)")
        .bind(user1)
        .bind(user2)
        .bind(user2)
        .bind(user1)
        .execute(&mut *tx)
        .await?;

    query("DELETE FROM FriendSettings WHERE (UserId = ? AND FriendId = ?) OR (UserId = ? AND FriendId = ?)")
        .bind(user1)
        .bind(user2)
        .bind(user2)
        .bind(user1)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_friend_settings(pool: &DatabasePool, user_id: UserId, friend_id: UserId) -> DatabaseResult<FriendSettings> {
    let settings = query("SELECT PermissionLevel, NotifyMe FROM FriendSettings WHERE UserId = ? AND FriendId = ?")
        .bind(user_id)
        .bind(friend_id)
        .fetch_optional(pool.as_ref())
        .await?;

    if let Some(settings) = settings {
        let permission_level: i8 = settings.get("PermissionLevel");
        let notify_me: bool = settings.get("NotifyMe");

        Ok(FriendSettings {
            permission_level,
            notify_me,
        })
    } else {
        Ok(FriendSettings::default())
    }
}

pub async fn update_friend_settings(pool: &DatabasePool, user_id: UserId, friend_id: UserId, settings: FriendSettings) -> DatabaseResult<()> {
    query("INSERT INTO FriendSettings (UserId, FriendId, PermissionLevel, NotifyMe) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE PermissionLevel = ?, NotifyMe = ?")
        .bind(user_id)
        .bind(friend_id)
        .bind(settings.permission_level)
        .bind(settings.notify_me)
        .bind(settings.permission_level)
        .bind(settings.notify_me)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}