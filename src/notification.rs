use serde_json::json;
use tokio::sync::OnceCell;
use web_push::{ContentEncoding, IsahcWebPushClient, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient, WebPushError, WebPushMessageBuilder};
use crate::database::DatabasePool;
use crate::error::WebError;
use crate::model::user::UserId;

const WEB_PUSH_VAPID_PRIVATE_KEY: &'static str = env!("WEB_PUSH_VAPID_PRIVATE_KEY_BASE64");
const WEB_PUSH_VAPID_SUB: &'static str = env!("WEB_PUSH_VAPID_SUB");

static WEB_PUSH_CLIENT: OnceCell<IsahcWebPushClient> = OnceCell::const_new();
static WEB_PUSH_TRUSTED_ENDPOINTS: OnceCell<Vec<&'static str>> = OnceCell::const_new();

async fn get_web_push_client() -> &'static IsahcWebPushClient {
    WEB_PUSH_CLIENT.get_or_init(|| async {
        IsahcWebPushClient::new().unwrap()
    }).await
}

async fn get_trusted_endpoints() -> &'static Vec<&'static str> {
    WEB_PUSH_TRUSTED_ENDPOINTS.get_or_init(|| async {
        env!("WEB_PUSH_TRUSTED_ENDPOINTS").split(",").collect()
    }).await
}

pub async fn check_endpoint(endpoint: &str) -> Result<(), WebError> {
    let endpoint = endpoint.strip_prefix("https://").ok_or(WebError::WebPushEndpointNotTrusted)?;
    let (endpoint, _) = endpoint.split_once("/").ok_or(WebError::WebPushEndpointNotTrusted)?;
    let trusted_endpoints = get_trusted_endpoints().await;
    for trusted_endpoint in trusted_endpoints {
        if *trusted_endpoint == endpoint {
            return Ok(());
        }
        if trusted_endpoint.starts_with("*") && endpoint.ends_with(&trusted_endpoint[1..]) {
            return Ok(());
        }
    }
    Err(WebError::WebPushEndpointNotTrusted)
}

pub async fn notify_user(
    pool: &DatabasePool,
    user_id: UserId,
    title: &str,
    body: &str,
    tag: &str,
) -> Result<(), anyhow::Error> {
    if cfg!(not(debug_assertions)) {
        let subscriptions = crate::database::notification::get_subscriptions(pool, user_id).await?;
        for subscription in subscriptions {
            if let Err(err) = send_push_notification(subscription.endpoint, subscription.keys.p256dh, subscription.keys.auth, title, body, tag).await {
                if matches!(err, WebPushError::InvalidUri | WebPushError::EndpointNotValid(_) | WebPushError::EndpointNotFound(_)) {
                    let _ = crate::database::notification::remove_subscription(pool, subscription.id).await?;
                }
            }
        }
    }
    Ok(())
}

async fn send_push_notification(
    endpoint: String,
    p256dh: String,
    auth: String,
    title: &str,
    body: &str,
    tag: &str,
) -> Result<(), WebPushError> {
    let subscription_info = SubscriptionInfo {
        endpoint,
        keys: SubscriptionKeys {
            p256dh,
            auth,
        },
    };

    let payload = json!({
        "title": title,
        "body": body,
        "tag": tag,
    });
    let payload = payload.to_string();

    let mut builder = WebPushMessageBuilder::new(&subscription_info);
    builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
    builder.set_ttl(600);

    let mut vapid = VapidSignatureBuilder::from_base64(WEB_PUSH_VAPID_PRIVATE_KEY, &subscription_info)?;
    vapid.add_claim("sub", WEB_PUSH_VAPID_SUB);
    let vapid = vapid.build()?;
    builder.set_vapid_signature(vapid);

    let request = builder.build()?;

    let client = get_web_push_client().await;
    client.send(request).await?;

    Ok(())
}