use actix_web::HttpResponse;
use serde::Serialize;
use crate::error::WebError;

pub mod auth;
pub mod version;
pub mod api;

pub(in crate::web) type WebResult = Result<HttpResponse, WebError>;

pub(in crate::web) fn ok(body: impl Serialize) -> WebResult {
    Ok(HttpResponse::Ok().json(body))
}

pub(in crate::web) fn ok_none() -> WebResult {
    Ok(HttpResponse::NoContent().finish())
}

pub(in crate::web) fn not_found() -> WebResult {
    Ok(HttpResponse::NotFound().finish())
}