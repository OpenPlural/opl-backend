use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::get_token;
use crate::model::front::{FrontCommentRequest, FrontEndTimeRequest, FrontEntry, FrontEntryId, FrontStartTimeRequest};
use crate::model::member::MemberId;
use crate::model::IdResponse;
use crate::web::{ok, ok_none, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, patch, put, HttpRequest};
use chrono::{DateTime, Utc};

#[put("/")]
pub async fn add_front_entry(req: HttpRequest, data: Data<AppState>, body: Json<FrontEntry>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let body = body.into_inner();
    let start_time = body.started_at;
    let start_time = DateTime::parse_from_rfc3339(start_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
    let start_time = start_time.with_timezone(&Utc);

    let user_id = crate::database::member::get_member_user_id(&data.pool, body.member).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    let end_time = if let Some(end_time) = body.ended_at {
        let end_time = DateTime::parse_from_rfc3339(end_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
        Some(end_time.with_timezone(&Utc))
    } else {
        None
    };

    let id = crate::database::front::add_front_entry_full(&data.pool, token.user_id, body.member, &start_time, end_time.as_ref(), &body.comment).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[put("/member/{id}")]
pub async fn front(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();
    let user_id = crate::database::member::get_member_user_id(&data.pool, member_id).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    if crate::database::front::check_fronting(&data.pool, member_id).await.map_err(to_web_error)? {
        return Err(WebError::AlreadyFronting)
    }

    let id = crate::database::front::add_front_entry(&data.pool, token.user_id, member_id).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[delete("/member/{id}")]
pub async fn unfront(req: HttpRequest, data: Data<AppState>, path: Path<MemberId>, query: Query<FrontEndTimeRequest>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let member_id = path.into_inner();
    let user_id = crate::database::member::get_member_user_id(&data.pool, member_id).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    let query = query.into_inner();
    let end_time = if let Some(end_time) = query.ended_at {
        let end_time = DateTime::parse_from_rfc3339(end_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
        end_time.with_timezone(&Utc)
    } else {
        Utc::now()
    };

    crate::database::front::end_current_front(&data.pool, member_id, &end_time).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}/comment")]
pub async fn front_comment(req: HttpRequest, data: Data<AppState>, body: Json<FrontCommentRequest>, path: Path<FrontEntryId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let entry_id = path.into_inner();
    let user_id = crate::database::front::get_front_user_id(&data.pool, entry_id).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    let body = body.into_inner();
    crate::database::front::edit_front_comment(&data.pool, entry_id, &body.comment).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}/startTime")]
pub async fn front_start_time(req: HttpRequest, data: Data<AppState>, body: Json<FrontStartTimeRequest>, path: Path<FrontEntryId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let entry_id = path.into_inner();
    let user_id = crate::database::front::get_front_user_id(&data.pool, entry_id).await.map_err(to_web_error)?;
    token.require_self(user_id)?;

    let body = body.into_inner();
    let start_time = body.started_at;
    let start_time = DateTime::parse_from_rfc3339(start_time.as_str()).map_err(|_| WebError::InvalidTimeFormat)?;
    let start_time = start_time.with_timezone(&Utc);
    crate::database::front::edit_start_time(&data.pool, entry_id, &start_time).await.map_err(to_web_error)?;
    ok_none()
}