use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::member::MemberId;
use crate::model::pluralkit::PkConfig;
use crate::model::user::UserId;

pub async fn get_pluralkit_config(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Option<PkConfig>> {
    let config = query("SELECT PkToken, PkDisplayName FROM User WHERE ID=?")
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(config.map(|row| {
        let token = row.get("PkToken");
        let display_name = row.get("PkDisplayName");
        PkConfig {
            token,
            display_name,
        }
    }))
}

pub async fn setup_pluralkit(pool: &DatabasePool, user_id: UserId, config: &PkConfig) -> DatabaseResult<()> {
    query("UPDATE User SET PkToken = ?, PkDisplayName = ? WHERE ID=?")
        .bind(&config.token)
        .bind(&config.display_name)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn assign_member_pk_id(pool: &DatabasePool, user_id: UserId, member_id: MemberId, pk_id: &str) -> DatabaseResult<()> {
    query("UPDATE Member SET PkId = ? WHERE ID=? AND UserId=?")
        .bind(pk_id)
        .bind(member_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;
    
    Ok(())
}