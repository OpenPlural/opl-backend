use sqlx::query;
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::user::UserId;

pub async fn update_password_reset_token(pool: &DatabasePool, user_id: UserId, token_hash: &str) -> DatabaseResult<()> {
    query("UPDATE User SET PasswordResetToken = ?, PasswordResetTokenExpires = TIMESTAMPADD(DAY, 1, CURRENT_TIMESTAMP()) WHERE ID = ?")
        .bind(token_hash)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}