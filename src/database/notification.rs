use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::notification::{PushSubscription, PushSubscriptionKeys};
use crate::model::session::SessionId;
use crate::model::user::UserId;

pub async fn add_subscription(pool: &DatabasePool, user_id: UserId, session_id: SessionId, push_subscription: &PushSubscription) -> DatabaseResult<()> {
    query("INSERT IGNORE INTO Notification (UserId, SessionId, Endpoint, p256dh, auth) VALUES (?, ?, ?, ?, ?)")
        .bind(user_id)
        .bind(session_id)
        .bind(&push_subscription.endpoint)
        .bind(&push_subscription.keys.p256dh)
        .bind(&push_subscription.keys.auth)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn set_last_notification(pool: &DatabasePool, fronting_user_id: UserId, receiving_user_id: UserId, front_text: &str) -> DatabaseResult<bool> {
    let res = query("SELECT 1 FROM LastNotification WHERE FrontingUserId = ? AND ReceivingUserId = ? AND FrontText = ?")
        .bind(fronting_user_id)
        .bind(receiving_user_id)
        .bind(front_text)
        .fetch_optional(pool.as_ref())
        .await?;
    if res.is_some() {
        return Ok(true);
    }
    query("INSERT INTO LastNotification (FrontingUserId, ReceivingUserId, FrontText) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE FrontText = ?")
        .bind(fronting_user_id)
        .bind(receiving_user_id)
        .bind(front_text)
        .bind(front_text)
        .execute(pool.as_ref())
        .await?;
    Ok(false)
}

pub async fn remove_subscription(pool: &DatabasePool, id: i64) -> DatabaseResult<()> {
    query("DELETE FROM Notification WHERE ID = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_subscriptions(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PushSubscription>> {
    let subscriptions = query("SELECT ID, Endpoint, p256dh, auth FROM Notification WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(subscriptions.into_iter().map(|row| PushSubscription {
        id: row.get("ID"),
        endpoint: row.get("Endpoint"),
        keys: PushSubscriptionKeys {
            p256dh: row.get("p256dh"),
            auth: row.get("auth"),
        },
    }).collect())
}