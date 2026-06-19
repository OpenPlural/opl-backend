use crate::database::{DatabaseExecutor, DatabasePool, DatabaseResult};
use crate::model::privacy::{PrivacyBucket, PrivacyBucketId, SimplePrivacyBucket};
use crate::model::user::UserId;
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};
use crate::model::fields::CustomFieldId;
use crate::model::folder::FolderId;
use crate::model::member::MemberId;

pub async fn get_privacy_buckets(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PrivacyBucket>> {
    let buckets = query("SELECT ID, UserId, Sort, Name, Description, Emoji, Color FROM PrivacyBucket WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;
    let mut buckets: Vec<PrivacyBucket> = buckets.into_iter().map(|row| bucket(row, vec![], vec![], vec![])).collect();

    if !buckets.is_empty() {
        let folders = {
            let placeholders = buckets.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
            let sql = format!("SELECT BucketId, FolderId FROM PrivacyBucketFolder WHERE BucketId IN ({placeholders})");
            let mut query = query(sql.as_str());
            for bucket in &buckets {
                query = query.bind(bucket.id);
            }
            query
                .fetch_all(pool.as_ref())
                .await?
        };
        let members = {
            let placeholders = buckets.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
            let sql = format!("SELECT BucketId, MemberId FROM PrivacyBucketMember WHERE BucketId IN ({placeholders})");
            let mut query = query(sql.as_str());
            for bucket in &buckets {
                query = query.bind(bucket.id);
            }
            query
                .fetch_all(pool.as_ref())
                .await?
        };
        let friends = {
            let placeholders = buckets.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
            let sql = format!("SELECT BucketId, FriendId FROM PrivacyBucketFriend WHERE BucketId IN ({placeholders})");
            let mut query = query(sql.as_str());
            for bucket in &buckets {
                query = query.bind(bucket.id);
            }
            query
                .fetch_all(pool.as_ref())
                .await?
        };

        for bucket in buckets.iter_mut() {
            bucket.folders = folders.iter()
                .filter(|row| row.get::<PrivacyBucketId, _>("BucketId") == bucket.id)
                .map(|row| row.get("FolderId"))
                .collect();
            bucket.members = members.iter()
                .filter(|row| row.get::<PrivacyBucketId, _>("BucketId") == bucket.id)
                .map(|row| row.get("MemberId"))
                .collect();
            bucket.friends = friends.iter()
                .filter(|row| row.get::<PrivacyBucketId, _>("BucketId") == bucket.id)
                .map(|row| row.get("FriendId"))
                .collect();
        }
    }

    Ok(buckets)
}

pub async fn get_privacy_bucket(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId) -> DatabaseResult<Option<PrivacyBucket>> {
    let res = query("SELECT ID, UserId, Sort, Name, Description, Emoji, Color FROM PrivacyBucket WHERE ID = ? AND UserId = ?")
        .bind(bucket_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;
    if res.is_none() {
        return Ok(None);
    }

    let folders = query("SELECT FolderId FROM PrivacyBucketFolder WHERE BucketId = ?")
        .bind(bucket_id)
        .fetch_all(pool.as_ref())
        .await?;
    let members = query("SELECT MemberId FROM PrivacyBucketMember WHERE BucketId = ?")
        .bind(bucket_id)
        .fetch_all(pool.as_ref())
        .await?;
    let friends = query("SELECT FriendId FROM PrivacyBucketFriend WHERE BucketId = ?")
        .bind(bucket_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(Some(bucket(res.unwrap(), folders, members, friends)))
}

pub async fn create_privacy_bucket<'a, E: DatabaseExecutor<'a>>(executor: E, bucket: &PrivacyBucket) -> DatabaseResult<PrivacyBucketId> {
    let id = query("INSERT INTO PrivacyBucket (UserId, Sort, Name, Description, Emoji) VALUES (?, ?, ?, ?, ?) RETURNING ID")
        .bind(bucket.user_id)
        .bind(bucket.sort)
        .bind(&bucket.name)
        .bind(&bucket.description)
        .bind(&bucket.emoji)
        .fetch_one(executor)
        .await?;

    Ok(id.get(0))
}

pub async fn delete_privacy_bucket(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM PrivacyBucket WHERE ID = ? AND UserId = ?")
        .bind(bucket_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_privacy_bucket(pool: &DatabasePool, bucket: &PrivacyBucket) -> DatabaseResult<()> {
    query("UPDATE PrivacyBucket SET Name = ?, Description = ?, Emoji = ?, Color = ? WHERE ID = ? AND UserId = ?")
        .bind(&bucket.name)
        .bind(&bucket.description)
        .bind(&bucket.emoji)
        .bind(bucket.color)
        .bind(bucket.id)
        .bind(bucket.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn reorder_privacy_buckets(pool: &DatabasePool, ids: Vec<PrivacyBucketId>, user_id: UserId) -> DatabaseResult<()> {
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
    let sql = format!("UPDATE PrivacyBucket SET Sort=field(ID, {placeholders}) WHERE ID IN ({placeholders}) AND UserId = ?");
    let mut query = query(sql.as_str());
    for id in &ids {
        query = query.bind(id);
    }
    for id in &ids {
        query = query.bind(id);
    }
    query
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;
    Ok(())
}

pub async fn add_privacy_bucket_folder<'a, E: DatabaseExecutor<'a>>(executor: E, bucket_id: PrivacyBucketId, user_id: UserId, folder_id: FolderId) -> DatabaseResult<()> {
    query("INSERT INTO PrivacyBucketFolder (UserId, BucketId, FolderId) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(bucket_id)
        .bind(folder_id)
        .execute(executor)
        .await?;

    Ok(())
}

pub async fn remove_privacy_bucket_folder(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId, folder_id: FolderId) -> DatabaseResult<()> {
    query("DELETE FROM PrivacyBucketFolder WHERE UserId = ? AND BucketId = ? AND FolderId = ?")
        .bind(user_id)
        .bind(bucket_id)
        .bind(folder_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_folder_privacy_buckets(pool: &DatabasePool, folder_id: FolderId, user_id: UserId) -> DatabaseResult<Vec<SimplePrivacyBucket>> {
    let res = query("SELECT ID, Sort, Name, Emoji, Color FROM PrivacyBucketFolder f JOIN PrivacyBucket p ON p.ID = f.BucketId WHERE f.FolderId = ? AND f.UserId = ?")
        .bind(folder_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(res.into_iter().map(simple_bucket).collect())
}

pub async fn add_privacy_bucket_member<'a, E: DatabaseExecutor<'a>>(executor: E, bucket_id: PrivacyBucketId, user_id: UserId, member_id: MemberId) -> DatabaseResult<()> {
    query("INSERT INTO PrivacyBucketMember (UserId, BucketId, MemberId) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(bucket_id)
        .bind(member_id)
        .execute(executor)
        .await?;

    Ok(())
}

pub async fn remove_privacy_bucket_member(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId, member_id: MemberId) -> DatabaseResult<()> {
    query("DELETE FROM PrivacyBucketMember WHERE UserId = ? AND BucketId = ? AND MemberId = ?")
        .bind(user_id)
        .bind(bucket_id)
        .bind(member_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_member_privacy_buckets(pool: &DatabasePool, member_id: MemberId, user_id: UserId) -> DatabaseResult<Vec<SimplePrivacyBucket>> {
    let res = query("SELECT ID, Sort, Name, Emoji, Color FROM PrivacyBucketMember m JOIN PrivacyBucket p ON p.ID = m.BucketId WHERE m.MemberId = ? AND m.UserId = ?")
        .bind(member_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(res.into_iter().map(simple_bucket).collect())
}

pub async fn add_privacy_bucket_custom_field<'a, E: DatabaseExecutor<'a>>(executor: E, bucket_id: PrivacyBucketId, user_id: UserId, field_id: CustomFieldId) -> DatabaseResult<()> {
    query("INSERT INTO PrivacyBucketCustomField (UserId, BucketId, FieldId) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(bucket_id)
        .bind(field_id)
        .execute(executor)
        .await?;

    Ok(())
}

pub async fn remove_privacy_bucket_custom_field(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId, field_id: CustomFieldId) -> DatabaseResult<()> {
    query("DELETE FROM PrivacyBucketCustomField WHERE UserId = ? AND BucketId = ? AND FieldId = ?")
        .bind(user_id)
        .bind(bucket_id)
        .bind(field_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_custom_field_privacy_buckets(pool: &DatabasePool, field_id: CustomFieldId, user_id: UserId) -> DatabaseResult<Vec<SimplePrivacyBucket>> {
    let res = query("SELECT ID, Sort, Name, Emoji, Color FROM PrivacyBucketCustomField c JOIN PrivacyBucket p ON p.ID = c.BucketId WHERE c.FieldId = ? AND c.UserId = ?")
        .bind(field_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(res.into_iter().map(simple_bucket).collect())
}

pub async fn add_privacy_bucket_friend(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId, friend_id: UserId) -> DatabaseResult<()> {
    query("INSERT INTO PrivacyBucketFriend (UserId, BucketId, FriendId) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(bucket_id)
        .bind(friend_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn remove_privacy_bucket_friend(pool: &DatabasePool, bucket_id: PrivacyBucketId, user_id: UserId, friend_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM PrivacyBucketFriend WHERE UserId = ? AND BucketId = ? AND FriendId = ?")
        .bind(user_id)
        .bind(bucket_id)
        .bind(friend_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_friend_privacy_buckets(pool: &DatabasePool, friend_id: UserId, user_id: UserId) -> DatabaseResult<Vec<SimplePrivacyBucket>> {
    let res = query("SELECT ID, Sort, Name, Emoji, Color FROM PrivacyBucketFriend f JOIN PrivacyBucket p ON p.ID = f.BucketId WHERE f.FriendId = ? AND f.UserId = ?")
        .bind(friend_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(res.into_iter().map(simple_bucket).collect())
}

pub async fn get_privacy_bucket_owner(pool: &DatabasePool, bucket_id: PrivacyBucketId) -> DatabaseResult<Option<UserId>> {
    let owner = query("SELECT UserId FROM PrivacyBucket WHERE ID = ?")
        .bind(bucket_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(owner.map(|row| row.get("UserId")))
}

fn bucket(row: MySqlRow, folders: Vec<MySqlRow>, members: Vec<MySqlRow>, friends: Vec<MySqlRow>) -> PrivacyBucket {
    let id = row.get("ID");
    let user_id = row.get("UserId");
    let sort = row.get("Sort");
    let name = row.get("Name");
    let description = row.get("Description");
    let emoji = row.get("Emoji");
    let color = row.get("Color");
    let folders = folders.into_iter().map(|row| row.get(0)).collect();
    let members = members.into_iter().map(|row| row.get(0)).collect();
    let friends = friends.into_iter().map(|row| row.get(0)).collect();

    PrivacyBucket {
        id,
        user_id,
        sort,
        name,
        description,
        emoji,
        color,
        folders,
        members,
        friends,
    }
}

fn simple_bucket(row: MySqlRow) -> SimplePrivacyBucket {
    let id = row.get("ID");
    let sort = row.get("Sort");
    let name = row.get("Name");
    let emoji = row.get("Emoji");
    let color = row.get("Color");

    SimplePrivacyBucket {
        id,
        sort,
        name,
        emoji,
        color,
    }
}