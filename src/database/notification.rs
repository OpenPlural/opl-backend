use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::notification::{FCMSubscription, PushSubscription, WebPushSubscription, WebPushSubscriptionKeys};
use crate::model::session::SessionId;
use crate::model::user::UserId;

pub async fn add_web_push_subscription(pool: &DatabasePool, user_id: UserId, session_id: SessionId, push_subscription: &WebPushSubscription) -> DatabaseResult<()> {
    query("INSERT INTO WebPushSubscription (UserId, SessionId, Endpoint, p256dh, auth) VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE Endpoint = VALUES(Endpoint), p256dh = VALUES(p256dh), auth = VALUES(auth)")
        .bind(user_id)
        .bind(session_id)
        .bind(&push_subscription.endpoint)
        .bind(&push_subscription.keys.p256dh)
        .bind(&push_subscription.keys.auth)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn add_fcm_subscription(pool: &DatabasePool, user_id: UserId, session_id: SessionId, push_subscription: &FCMSubscription) -> DatabaseResult<()> {
    query("INSERT INTO FCMSubscription (UserId, SessionId, InstallationId) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE InstallationId = VALUES(InstallationId)")
        .bind(user_id)
        .bind(session_id)
        .bind(&push_subscription.installation_id)
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
        return Ok(false);
    }
    query("INSERT INTO LastNotification (FrontingUserId, ReceivingUserId, FrontText) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE FrontText = ?")
        .bind(fronting_user_id)
        .bind(receiving_user_id)
        .bind(front_text)
        .bind(front_text)
        .execute(pool.as_ref())
        .await?;
    Ok(true)
}

pub async fn clear_last_notification(pool: &DatabasePool, fronting_user_id: UserId, receiving_user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM LastNotification WHERE FrontingUserId = ? AND ReceivingUserId = ?")
        .bind(fronting_user_id)
        .bind(receiving_user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn remove_web_push_subscription(pool: &DatabasePool, id: i64) -> DatabaseResult<()> {
    query("DELETE FROM WebPushSubscription WHERE ID = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn remove_fcm_subscription(pool: &DatabasePool, id: i64) -> DatabaseResult<()> {
    query("DELETE FROM FCMSubscription WHERE ID = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_subscriptions(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PushSubscription>> {
    let subscriptions = query("SELECT ID, 'WebPush' as Type, Endpoint, p256dh, auth FROM WebPushSubscription WHERE UserId = ? UNION SELECT ID, 'FCM' as Type, InstallationId AS Endpoint, '' as p256dh, '' as auth FROM FCMSubscription WHERE UserId = ?")
        .bind(user_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(subscriptions.into_iter().map(|row| {
        let r#type = row.get("Type");
        match r#type {
            "WebPush" => PushSubscription::WebPush(WebPushSubscription {
                id: row.get("ID"),
                endpoint: row.get("Endpoint"),
                keys: WebPushSubscriptionKeys {
                    p256dh: row.get("p256dh"),
                    auth: row.get("auth"),
                },
            }),
            "FCM" => PushSubscription::FCM(FCMSubscription {
                id: row.get("ID"),
                installation_id: row.get("Endpoint")
            }),
            _ => unreachable!()
        }
    }).collect())
}