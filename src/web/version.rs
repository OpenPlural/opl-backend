use std::str::FromStr;
use actix_web::get;
use serde_json::json;
use crate::web::{ok, WebResult};

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