use actix_web::get;
use serde_json::json;

#[get("/version")]
pub async fn version() -> String {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "details": {
            "major": env!("CARGO_PKG_VERSION_MAJOR"),
            "minor": env!("CARGO_PKG_VERSION_MINOR"),
            "patch": env!("CARGO_PKG_VERSION_PATCH")
        }
    }).to_string()
}

#[get("/app_update")]
pub async fn app_update() -> &'static str {
    "Indev"
}