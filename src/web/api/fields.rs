use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::fields::{CustomField, CustomFieldDataId, CustomFieldDataValue, CustomFieldId};
use crate::model::IdResponse;
use crate::web::{not_found, ok, ok_none, validation_error, WebResult};
use crate::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, patch, post, put, HttpRequest};

#[get("/")]
pub async fn get_fields(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();

    let fields = crate::database::fields::get_fields(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(fields)
}

#[get("/{id}")]
pub async fn get_field(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldId>) -> WebResult {
    let token = get_token(&req).unwrap();
    let field_id = path.into_inner();

    if let Some(field) = crate::database::fields::get_field_by_id(&data.pool, field_id, token.user_id).await.map_err(to_web_error)? {
        ok(field)
    } else {
        not_found()
    }
}

#[put("/")]
pub async fn create_field(req: HttpRequest, data: Data<AppState>, body: Json<CustomField>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    let id = crate::database::fields::create_field(&data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[delete("/{id}")]
pub async fn delete_field(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let field_id = path.into_inner();
    crate::database::fields::delete_field(&data.pool, field_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[patch("/{id}")]
pub async fn edit_field(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldId>, body: Json<CustomField>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let field_id = path.into_inner();
    body.id = field_id;
    body.user_id = token.user_id;

    crate::database::fields::edit_field(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[post("/reorder")]
pub async fn reorder_fields(req: HttpRequest, data: Data<AppState>, body: Json<Vec<CustomFieldId>>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let body = body.into_inner();
    crate::database::fields::reorder_fields(&data.pool, body, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/values")]
pub async fn get_specific_field_values(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldId>) -> WebResult {
    let token = get_token(&req).unwrap();
    let field_id = path.into_inner();

    let values = crate::database::fields::get_field_values_for_field(&data.pool, token.user_id, field_id).await.map_err(to_web_error)?;
    ok(values)
}

#[get("/value/id")]
pub async fn get_field_values(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();

    let fields = crate::database::fields::get_field_value_ids(&data.pool, token.user_id).await.map_err(to_web_error)?;
    ok(fields)
}

#[get("/value/{id}")]
pub async fn get_field_value(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldDataId>) -> WebResult {
    let token = get_token(&req).unwrap();
    let value_id = path.into_inner();

    if let Some(value) = crate::database::fields::get_field_value_by_id(&data.pool, value_id, token.user_id).await.map_err(to_web_error)? {
        ok(value)
    } else {
        not_found()
    }
}

#[put("/value/")]
pub async fn create_field_value(req: HttpRequest, data: Data<AppState>, body: Json<CustomFieldDataValue>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;
    body.user_id = token.user_id;

    let id = crate::database::fields::create_field_value(&data.pool, &body).await.map_err(to_web_error)?;
    ok(IdResponse {
        id
    })
}

#[patch("/value/{id}")]
pub async fn update_field_value(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldDataId>, body: Json<CustomFieldDataValue>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let mut body = body.into_inner();
    body.validate().map_err(validation_error)?;

    let data_id = path.into_inner();
    body.id = data_id;
    body.user_id = token.user_id;

    crate::database::fields::update_field_value(&data.pool, &body).await.map_err(to_web_error)?;
    ok_none()
}

#[delete("/value/{id}")]
pub async fn clear_field_value(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldDataId>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_write()?;

    let data_id = path.into_inner();
    crate::database::fields::clear_field_value(&data.pool, data_id, token.user_id).await.map_err(to_web_error)?;
    ok_none()
}

#[get("/{id}/privacy")]
pub async fn get_field_privacy(req: HttpRequest, data: Data<AppState>, path: Path<CustomFieldId>) -> WebResult {
    let token = get_token(&req).unwrap();

    let field_id = path.into_inner();

    let buckets = crate::database::privacy::get_custom_field_privacy_buckets(&data.pool, field_id, token.user_id).await.map_err(to_web_error)?;
    ok(buckets)
}