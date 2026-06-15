use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::{Mutex, OnceCell};
use tokio::time::{interval, Instant};
use crate::database::DatabasePool;
use crate::model::user::UserId;

static FRONT_UPDATES: OnceCell<Arc<Mutex<HashMap<UserId, Instant>>>> = OnceCell::const_new();

const NOTIFY_AFTER_DELAY: Duration = Duration::from_secs(10);
const MAX_FRONT_TEXT_LENGTH: usize = 254;

async fn get_front_updates() -> &'static Arc<Mutex<HashMap<UserId, Instant>>> {
    FRONT_UPDATES.get_or_init(|| async {
        Arc::new(Mutex::new(HashMap::new()))
    }).await
}

pub async fn notify_front_change(user_id: UserId) {
    let front_updates = get_front_updates().await;
    let mut front_updates = front_updates.lock().await;
    front_updates.insert(user_id, Instant::now());
}

pub async fn watch_front_changes(database_pool: DatabasePool) {
    spawn(async move {
        let front_updates = get_front_updates().await;
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;

            let mut notify_users = vec![];
            {
                let mut front_updates = front_updates.lock().await;
                for (user_id, update_time) in front_updates.iter() {
                    if update_time.elapsed() >= NOTIFY_AFTER_DELAY {
                        notify_users.push(*user_id);
                    }
                }
                for user_id in &notify_users {
                    front_updates.remove(user_id);
                }
            }

            for user_id in notify_users {
                if let Ok(Some(username)) = crate::database::user::get_username(&database_pool, user_id).await {
                    if let Ok(notified_friends) = crate::database::friend::get_notified_friend_ids(&database_pool, user_id).await {
                        if let Ok(front_text) = crate::database::front::get_notification_front_text(&database_pool, notified_friends, user_id).await {
                            for (notify_user, front_text) in front_text {
                                if let Some(mut front_text) = front_text {
                                    if front_text.len() > MAX_FRONT_TEXT_LENGTH {
                                        front_text.truncate(front_text.floor_char_boundary(MAX_FRONT_TEXT_LENGTH));
                                        front_text.push('…');
                                    }
                                    if let Ok(true) = crate::database::notification::set_last_notification(&database_pool, user_id, notify_user, &front_text).await {
                                        let _ = crate::notification::notify_user(&database_pool, notify_user, &username, &front_text, "front-change").await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}