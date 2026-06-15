use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PushSubscription {
    #[serde(skip)]
    pub id: i64,
    pub endpoint: String,
    pub keys: PushSubscriptionKeys
}

#[derive(Debug, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}