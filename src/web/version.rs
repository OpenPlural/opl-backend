use crate::web::{ok, WebResult};
use actix_web::{get, HttpResponse};
use serde_json::json;
use std::env::var;
use std::str::FromStr;

#[get("/app-update")]
pub async fn app_update() -> HttpResponse {
    if let Ok(ver) = var("APP_VERSION") {
        HttpResponse::Ok().body(ver)
    } else {
        HttpResponse::NotImplemented().finish()
    }
}

#[get("/version")]
pub async fn version() -> WebResult {
    ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "details": {
            "major": usize::from_str(env!("CARGO_PKG_VERSION_MAJOR")).unwrap(),
            "minor": usize::from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap(),
            "patch": usize::from_str(env!("CARGO_PKG_VERSION_PATCH")).unwrap(),
        }
    }))
}