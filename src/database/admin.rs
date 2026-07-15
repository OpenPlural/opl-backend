use crate::database::{DatabasePool, DatabaseResult};
use crate::model::admin::{AdminStatisticsResponse, AdminUserListEntry};
use crate::model::user::UserId;
use chrono::Utc;
use sqlx::{query, Row};

pub async fn update_password_reset_token(pool: &DatabasePool, user_id: UserId, token_hash: &str) -> DatabaseResult<()> {
    query("UPDATE User SET PasswordResetToken = ?, PasswordResetTokenExpires = TIMESTAMPADD(DAY, 1, CURRENT_TIMESTAMP()) WHERE ID = ?")
        .bind(token_hash)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn clear_password_reset_token(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<()> {
    query("UPDATE User SET PasswordResetToken=NULL, PasswordResetTokenExpires=NULL WHERE ID = ?")
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_statistics(pool: &DatabasePool) -> DatabaseResult<AdminStatisticsResponse> {
    let stats = query(r#"
SELECT
    (SELECT COUNT(*) FROM User) AS Users,
    (SELECT COUNT(*) FROM Folder) AS Folders,
    (SELECT COUNT(*) FROM Member) AS Members,
    (SELECT COUNT(*) FROM CustomField) AS Fields,
    (SELECT COUNT(*) FROM CustomFieldData) AS FieldEntries,
    (SELECT COUNT(*) FROM Front) AS Fronts,
    (SELECT COUNT(*) FROM (SELECT DISTINCT MemberId FROM Front where EndedAt IS NULL) f) AS FrontMembers,
    (SELECT COUNT(*) FROM Friend) AS Friends,
    (SELECT COUNT(*) FROM Deletion) AS Deletions,
    (SELECT COUNT(*) FROM PrivacyBucket) AS PrivacyBuckets,
    (SELECT COUNT(*) FROM Session) AS Sessions,
    (SELECT COUNT(*) FROM (SELECT DISTINCT UserId FROM Session) s) AS SessionUsers,
    (SELECT COUNT(*) FROM ApiKey) AS ApiKeys,
    CURRENT_TIMESTAMP() AS Time
"#)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(AdminStatisticsResponse {
        users: stats.get("Users"),
        folders: stats.get("Folders"),
        members: stats.get("Members"),
        fields: stats.get("Fields"),
        field_entries: stats.get("FieldEntries"),
        fronts: stats.get("Fronts"),
        front_members: stats.get("FrontMembers"),
        friends: stats.get::<i64, _>("Friends") / 2,
        deletions: stats.get("Deletions"),
        privacy_buckets: stats.get("PrivacyBuckets"),
        sessions: stats.get("Sessions"),
        session_users: stats.get("SessionUsers"),
        api_keys: stats.get("ApiKeys"),
        database_time: stats.get("Time"),
        server_time: Utc::now()
    })
}


pub async fn get_users(pool: &DatabasePool) -> DatabaseResult<Vec<AdminUserListEntry>> {
    let users = query("SELECT ID, Name, Email, Color, System FROM User")
        .fetch_all(pool.as_ref())
        .await?;

    Ok(users.into_iter().map(|user| {
        let id = user.get("ID");
        let name = user.get("Name");
        let email = user.get("Email");
        let color = user.get("Color");
        let system = user.get("System");

        AdminUserListEntry {
            id,
            name,
            email,
            color,
            system,
        }
    }).collect())
}