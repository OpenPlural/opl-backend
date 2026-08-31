use chrono::{DateTime, Utc};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::analytics::{Analytics, AnalyticsMember};
use crate::model::user::UserId;
use sqlx::{query, Row};

pub async fn get_analytics(pool: &DatabasePool, user_id: UserId, start: DateTime<Utc>, end: DateTime<Utc>) -> DatabaseResult<Analytics> {
    let members = query("SELECT MemberId, COUNT(*) AS FrontCount, SUM(TIMESTAMPDIFF(MINUTE, StartedAt, IFNULL(EndedAt, CURRENT_TIMESTAMP()))) AS FrontMinutes FROM Front WHERE UserId = ? AND StartedAt >= ? AND (EndedAt IS NULL OR EndedAt <= ?) GROUP BY MemberId")
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(pool.as_ref())
        .await?;

    let member_analytics = members.into_iter().map(|row| {
        let id = row.get("MemberId");
        let front_count = row.get("FrontCount");
        let front_minutes = row.get("FrontMinutes");
        AnalyticsMember {
            id,
            front_count,
            front_minutes,
        }
    }).collect();
    Ok(Analytics {
        members: member_analytics
    })
}