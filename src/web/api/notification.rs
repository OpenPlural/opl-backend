use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::notification::PushSubscription;
use crate::web::{ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpRequest};

#[post("/subscribe")]
pub async fn subscribe(req: HttpRequest, data: Data<AppState>, body: Json<PushSubscription>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_session()?;

    let subscription = body.into_inner();
    crate::notification::check_endpoint(&subscription.endpoint).await?;
    crate::database::notification::add_subscription(&data.pool, token.user_id, token.session_id.unwrap(), &subscription).await.map_err(to_web_error)?;
    ok_none()
}