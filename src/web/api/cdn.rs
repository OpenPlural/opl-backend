use actix_web::{get, post, HttpRequest, HttpResponse};
use actix_web::web::{Bytes, Path, Query};
use uuid::Uuid;
use crate::database::cdn::{get_image, has_image, store_image};
use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::{get_token, RequestToken};
use crate::model::cdn::{AvatarAccessQuery, UploadResponse};
use crate::security::sha512_url;
use crate::web::{ok, WebResult};

const CDN_ACCESS_SECRET: &'static str = env!("CDN_ACCESS_SECRET");

fn get_access_token(id: &Uuid) -> String {
    sha512_url(format!("{}:{}", CDN_ACCESS_SECRET, id.as_simple()).as_str())
}

#[post("/upload")]
pub async fn upload_avatar(req: HttpRequest, body: Bytes) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let id = store_image(body).await.map_err(to_web_error)?;

    let access = get_access_token(&id);
    let id = id.simple().to_string();
    ok(UploadResponse {
        id,
        access,
    })
}

#[get("/{id}.{mimeType}")]
pub async fn get_avatar(req: HttpRequest, path: Path<(String, String)>, query: Query<AvatarAccessQuery>) -> WebResult {
    let token: RequestToken = get_token(&req).unwrap();
    token.require_session()?;

    let (id, mime_type) = path.into_inner();
    let id = Uuid::parse_str(&id).map_err(|err| WebError::InvalidPayload(format!("Invalid uuid: {err}")))?;

    let access = get_access_token(&id);
    if access != query.access {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if !has_image(&id).await.map_err(to_web_error)? {
        return Ok(HttpResponse::NotFound().finish());
    }
    let data = get_image(&id).await.map_err(to_web_error)?;

    let mut resp = HttpResponse::Ok();
    resp.content_type(format!("image/{mime_type}"));
    resp.insert_header(("Content-Disposition", format!("inline; filename={id}.{mime_type}")));
    Ok(resp.body(data))
}