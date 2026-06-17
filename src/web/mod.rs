use actix_web::HttpResponse;
use anyhow::anyhow;
use serde::Serialize;
use crate::error::WebError;
use crate::middleware::make_session_cookie;

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

pub(in crate::web) fn validation_error(msg: String) -> WebError {
    WebError::InvalidPayload(msg)
}

pub(in crate::web) fn ok_none_unauth() -> WebResult {
    let cookie = make_session_cookie("".to_string());

    let mut res = HttpResponse::NoContent().finish();
    res.add_removal_cookie(&cookie).map_err(|err| WebError::CantSetCookie(anyhow!("{:?}", err)))?;
    Ok(res)
}