use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::notification::{FCMSubscription, WebPushSubscription};
use crate::web::{ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpRequest};

#[post("/web")]
pub async fn subscribe_web_push(req: HttpRequest, data: Data<AppState>, body: Json<WebPushSubscription>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_real_session()?;

    let subscription = body.into_inner();
    crate::notification::check_endpoint(&subscription.endpoint).await?;
    crate::database::notification::add_web_push_subscription(&data.pool, token.user_id, token.session_id.unwrap(), &subscription).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/fcm")]
pub async fn subscribe_fcm(req: HttpRequest, data: Data<AppState>, body: Json<FCMSubscription>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_real_session()?;

    let subscription = body.into_inner();
    crate::database::notification::add_fcm_subscription(&data.pool, token.user_id, token.session_id.unwrap(), &subscription).await.map_err(to_web_error)?;
    ok_none()
}