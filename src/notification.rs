use chrono::Utc;
use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::sync::OnceCell;
use web_push::{ContentEncoding, IsahcWebPushClient, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient, WebPushError, WebPushMessageBuilder};
use crate::database::DatabasePool;
use crate::error::WebError;
use crate::model::notification::PushSubscription;
use crate::model::user::UserId;

const WEB_PUSH_VAPID_PRIVATE_KEY: &'static str = env!("WEB_PUSH_VAPID_PRIVATE_KEY_BASE64");
const WEB_PUSH_VAPID_SUB: &'static str = env!("WEB_PUSH_VAPID_SUB");
const FCM_PROJECT_ID: &'static str = env!("FCM_PROJECT_ID");

static WEB_PUSH_CLIENT: OnceCell<IsahcWebPushClient> = OnceCell::const_new();
static WEB_PUSH_TRUSTED_ENDPOINTS: OnceCell<Vec<&'static str>> = OnceCell::const_new();
static REQWEST_CLIENT: OnceCell<Client> = OnceCell::const_new();

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

async fn get_reqwest_client() -> &'static Client {
    REQWEST_CLIENT.get_or_init(|| async {
        Client::new()
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
    kind: &str,
    tag: Option<String>,
    fcm_token: &str,
) -> Result<(), anyhow::Error> {
    let id = (Utc::now().timestamp() - 1700000000) as i32;
    let subscriptions = crate::database::notification::get_subscriptions(pool, user_id).await?;
    for subscription in subscriptions {
        match subscription {
            PushSubscription::WebPush(subscription) => {
                if let Err(err) = send_web_push_notification(subscription.endpoint, subscription.keys.p256dh, subscription.keys.auth, title, body, kind, &tag).await {
                    if matches!(err, WebPushError::InvalidUri | WebPushError::EndpointNotValid(_) | WebPushError::EndpointNotFound(_)) {
                        let _ = crate::database::notification::remove_web_push_subscription(pool, subscription.id).await;
                    }
                }
            }
            PushSubscription::FCM(subscription) => {
                if let Err(err_code) = send_fcm_notification(fcm_token, subscription.installation_id, title, body, &tag, id).await {
                    if let Some(err_code) = err_code && err_code.as_u16() == 404 {
                        let _ = crate::database::notification::remove_fcm_subscription(pool, subscription.id).await;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn send_web_push_notification(
    endpoint: String,
    p256dh: String,
    auth: String,
    title: &str,
    body: &str,
    kind: &str,
    tag: &Option<String>,
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
        "kind": kind,
    });
    let payload = payload.to_string();

    let mut builder = WebPushMessageBuilder::new(&subscription_info);
    builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
    builder.set_ttl(1800);

    let mut vapid = VapidSignatureBuilder::from_base64(WEB_PUSH_VAPID_PRIVATE_KEY, &subscription_info)?;
    vapid.add_claim("sub", WEB_PUSH_VAPID_SUB);
    let vapid = vapid.build()?;
    builder.set_vapid_signature(vapid);

    let request = builder.build()?;

    let client = get_web_push_client().await;
    client.send(request).await?;

    Ok(())
}

pub async fn send_fcm_notification(
    fcm_token: &str,
    installation_id: String,
    title: &str,
    body: &str,
    tag: &Option<String>,
    id: i32,
) -> Result<(), Option<StatusCode>> {
    let id = Some(id).filter(|_| tag.is_some());
    let request = json!({
        "message": {
            "fid": installation_id,
            "notification": {
                "title": title,
                "body": body,
            },
            "android_v2": {
                "ttl": "1800s",
                "bandwidth_constrained_ok": true,
                "restricted_satellite_ok": true,
                "remote_notification": {
                    "mutable_content": false,
                    "notification": {
                        "title": title,
                        "body": body,
                        "sound": "default",
                        "tag": tag,
                        "id": id,
                    }
                }
            }
        }
    });

    let client = get_reqwest_client().await;
    let res = client.post(format!("https://fcm.googleapis.com/v1/projects/{FCM_PROJECT_ID}/messages:send"))
        .json(&request)
        .header("Authorization", fcm_token)
        .send()
        .await
        .map_err(|err| err.status())?;
    let status = res.status();
    if !status.is_success() {
        return Err(Some(status));
    }

    Ok(())
}