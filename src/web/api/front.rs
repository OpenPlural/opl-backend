use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::get_token;
use crate::model::folder::FolderId;
use crate::model::front::{FrontEntry, FrontEntryId};
use crate::web::{ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, patch, put, HttpRequest};
use chrono::{DateTime, Utc};

#[put("/{id}")]
pub async fn add_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<FolderId>, body: Json<FrontEntry>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let entry_id = path.into_inner();
    if crate::database::front::get_front_entry_by_id(&data.pool, entry_id, token.user_id).await.map_err(to_web_error)?.is_some() {
        return Err(WebError::IdDuplicate);
    }

    let mut body = body.into_inner();
    if body.id != entry_id {
        return Err(WebError::IdMismatch);
    }
    body.user = token.user_id;
    let start_time = &body.started_at;
    let start_time = DateTime::parse_from_rfc3339(start_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
    let start_time = start_time.with_timezone(&Utc);

    let end_time = if let Some(end_time) = &body.ended_at {
        let end_time = DateTime::parse_from_rfc3339(end_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
        Some(end_time.with_timezone(&Utc))
    } else {
        None
    };

    crate::database::front::add_front_entry(&data.pool, &body, start_time, end_time).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/{id}")]
pub async fn delete_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<FrontEntryId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let entry_id = path.into_inner();
    crate::database::front::delete_front_entry(&data.pool, entry_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_front_entry(req: HttpRequest, data: Data<AppState>, path: Path<FrontEntryId>, body: Json<FrontEntry>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let entry_id = path.into_inner();

    let mut body = body.into_inner();
    if body.id != entry_id {
        return Err(WebError::IdMismatch);
    }
    body.user = token.user_id;
    let start_time = &body.started_at;
    let start_time = DateTime::parse_from_rfc3339(start_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
    let start_time = start_time.with_timezone(&Utc);

    let end_time = if let Some(end_time) = &body.ended_at {
        let end_time = DateTime::parse_from_rfc3339(end_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
        Some(end_time.with_timezone(&Utc))
    } else {
        None
    };

    crate::database::front::edit_front_entry(&data.pool, &body, start_time, end_time).await.map_err(to_web_error)?;
    ok_none()
}