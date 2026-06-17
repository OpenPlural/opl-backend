use chrono::{DateTime, Utc};
use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};

pub async fn get_database_time(pool: &DatabasePool) -> DatabaseResult<DateTime<Utc>> {
    let time = query("SELECT CURRENT_TIMESTAMP()")
        .fetch_one(pool.as_ref())
        .await?;

    Ok(time.get(0))
}