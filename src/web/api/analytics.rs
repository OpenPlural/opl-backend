use actix_web::{get, HttpRequest};
use actix_web::web::{Data, Query};
use chrono::{NaiveTime, TimeDelta};
use crate::AppState;
use crate::database::to_web_error;
use crate::error::WebError;
use crate::middleware::get_token;
use crate::model::analytics::Analytics;
use crate::model::DateRangeQuery;
use crate::web::{ok, WebResult};

const ANALYTICS_MAX_DATE_RANGE: TimeDelta = TimeDelta::days(31);

#[get("/")]
pub async fn get_analytics(req: HttpRequest, data: Data<AppState>, query: Query<DateRangeQuery>) -> WebResult {
    let token = get_token(&req).unwrap();

    let start = query.start.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()).and_utc();
    let end = query.end.and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap()).and_utc();

    if end < start {
        return ok(Analytics {
            member: vec![]
        });
    }
    if end.signed_duration_since(&start) > ANALYTICS_MAX_DATE_RANGE {
        return Err(WebError::DateRangeTooBig("31 days"));
    }

    let analytics = crate::database::analytics::get_analytics(&data.pool, token.user_id, start, end).await.map_err(to_web_error)?;
    ok(analytics)
}