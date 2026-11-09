use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WebPushSubscription {
    #[serde(skip)]
    pub id: i64,
    pub endpoint: String,
    pub keys: WebPushSubscriptionKeys
}

#[derive(Debug, Deserialize)]
pub struct WebPushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct FCMSubscription {
    #[serde(skip)]
    pub id: i64,
    #[serde(rename = "installationId")]
    pub installation_id: String,
}

#[derive(Debug)]
pub enum PushSubscription {
    WebPush(WebPushSubscription),
    FCM(FCMSubscription),
}