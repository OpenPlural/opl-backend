use actix_web::{delete, get, patch, put, HttpRequest};
use actix_web::web::{Data, Json, Path};
use crate::AppState;
use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::get_token;
use crate::model::IdResponse;
use crate::model::poll::{Poll, PollAnswer, PollAnswerId, PollId};
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};

#[get("/")]
pub async fn get_polls(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();

    let polls = crate::database::poll::get_polls(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(polls)
}

#[get("/{id}")]
pub async fn get_poll(req: HttpRequest, data: Data<AppState>, path: Path<PollId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let poll_id = path.into_inner();

    if let Some(poll) = crate::database::poll::get_poll_by_id(&data.pool, poll_id, token.user_id).await.map_err(to_web_error)? {
        ok(poll)
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn create_poll(req: HttpRequest, data: Data<AppState>, body: Json<Poll>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    let id = crate::database::poll::create_poll(&data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[delete("/{id}")]
pub async fn delete_poll(req: HttpRequest, data: Data<AppState>, path: Path<PollId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let poll_id = path.into_inner();
    crate::database::poll::delete_poll(&data.pool, poll_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_poll(req: HttpRequest, data: Data<AppState>, path: Path<PollId>, body: Json<Poll>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;


    let poll_id = path.into_inner();
    body.id = poll_id;
    body.user_id = token.user_id;

    let old_custom = crate::database::poll::is_custom_poll(&data.pool, poll_id, token.user_id).await.map_err(to_web_error)?;
    if let Some(old_custom) = old_custom {
        let new_custom = body.custom_options.is_some();
        if old_custom != new_custom {
            return Err(WebError::CantChangePollType);
        }
    } else {
        return not_found();
    }

    crate::database::poll::edit_poll(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/answers")]
pub async fn get_poll_answers(req: HttpRequest, data: Data<AppState>, path: Path<PollId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let poll_id = path.into_inner();

    let polls = crate::database::poll::get_poll_answers(&data.pool, poll_id, token.user_id).await.map_err(to_web_error)?;
    ok(polls)
}

#[put("/answer/")]
pub async fn create_poll_answer(req: HttpRequest, data: Data<AppState>, body: Json<PollAnswer>) -> WebResult {
    let token = get_token(&req).unwrap();

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    let id = crate::database::poll::create_poll_answer(&data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[delete("/answer/{id}")]
pub async fn delete_poll_answer(req: HttpRequest, data: Data<AppState>, path: Path<PollAnswerId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let answer_id = path.into_inner();
    crate::database::poll::delete_poll_answer(&data.pool, answer_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/answer/{id}")]
pub async fn edit_poll_answer(req: HttpRequest, data: Data<AppState>, path: Path<PollAnswerId>, body: Json<PollAnswer>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let answer_id = path.into_inner();
    body.id = answer_id;
    body.user_id = token.user_id;

    crate::database::poll::edit_poll_answer(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}