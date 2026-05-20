use crate::database::{DatabasePool, DatabaseResult};
use crate::model::deletion::{Deletion, DeletionResourceType};
use crate::model::user::UserId;
use sqlx::{query, Row};

pub async fn get_deletions(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<Deletion>> {
    let deletions = query("SELECT ResourceId, ResourceType FROM Deletion WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(deletions.into_iter().map(|row| {
        let resource_id = row.get("ResourceId");
        let resource_type = row.get("ResourceType");
        let resource_type = DeletionResourceType::from_repr(resource_type).unwrap();
        Deletion {
            resource_id,
            resource_type,
        }
    }).collect())
}

pub async fn clear_old_deletions(pool: &DatabasePool) -> DatabaseResult<()> {
    query("DELETE FROM Deletion WHERE ValidUntil < CURRENT_TIMESTAMP()")
        .execute(pool.as_ref())
        .await?;

    Ok(())
}