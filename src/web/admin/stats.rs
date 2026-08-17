use actix_web::{get, HttpRequest};
use actix_web::web::Data;
use crate::AppState;
use crate::database::to_web_error;
use crate::web::admin::verify_admin_token;
use crate::web::{ok, WebResult};

#[get("/")]
pub async fn get_statistics(req: HttpRequest, data: Data<AppState>) -> WebResult {
    verify_admin_token(&req)?;

    let stats = crate::database::admin::get_statistics(&data.pool).await.map_err(to_web_error)?;
    ok(stats)
}