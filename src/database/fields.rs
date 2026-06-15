use crate::database::{DatabasePool, DatabaseResult};
use crate::model::fields::{CustomField, CustomFieldDataId, CustomFieldDataType, CustomFieldDataValue, CustomFieldId, ViewedCustomFieldDataValue};
use crate::model::member::MemberId;
use crate::model::user::UserId;
use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};

pub async fn get_field_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<CustomFieldId>> {
    let ids = query("SELECT ID FROM CustomField WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_updated_fields(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<CustomField>> {
    let updated = query("SELECT ID, UserId, Sort, Name, DataType, UpdatedAt FROM CustomField WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(updated.into_iter().map(field).collect())
}

pub async fn get_fields(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<CustomField>> {
    let fields = query("SELECT ID, UserId, Sort, Name, DataType, UpdatedAt FROM CustomField WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(fields.into_iter().map(field).collect())
}

pub async fn get_field_by_id(pool: &DatabasePool, field_id: CustomFieldId, user_id: UserId) -> DatabaseResult<Option<CustomField>> {
    let res = query("SELECT ID, UserId, Sort, Name, DataType, UpdatedAt FROM CustomField WHERE ID = ? AND UserId = ?")
        .bind(field_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(res.map(field))
}

pub async fn create_field(pool: &DatabasePool, field: &CustomField) -> DatabaseResult<CustomFieldId> {
    let id = query("INSERT INTO CustomField (UserId, Sort, Name, DataType) VALUES (?, ?, ?, ?) RETURNING ID")
        .bind(field.user_id)
        .bind(field.sort)
        .bind(&field.name)
        .bind(field.data_type as u8)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn delete_field(pool: &DatabasePool, field_id: CustomFieldId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM CustomField WHERE ID = ? AND UserId = ?")
        .bind(field_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_field(pool: &DatabasePool, field: &CustomField) -> DatabaseResult<()> {
    query("UPDATE CustomField SET Sort = ?, Name = ?, DataType = ? WHERE ID = ? AND UserId = ?")
        .bind(field.sort)
        .bind(&field.name)
        .bind(field.data_type as u8)
        .bind(field.id)
        .bind(field.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn reorder_fields(pool: &DatabasePool, ids: Vec<CustomFieldId>, user_id: UserId) -> DatabaseResult<()> {
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
    let sql = format!("UPDATE CustomField SET Sort=field(ID, {placeholders}) WHERE ID IN ({placeholders}) AND UserId = ?");
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

pub async fn get_field_value_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<CustomFieldDataId>> {
    let ids = query("SELECT ID FROM CustomFieldData WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_updated_field_values(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<CustomFieldDataValue>> {
    let updated = query("SELECT ID, UserId, FieldId, MemberId, DataValue, UpdatedAt FROM CustomFieldData WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(updated.into_iter().map(field_value).collect())
}

pub async fn get_field_values_for_field(pool: &DatabasePool, user_id: UserId, field_id: CustomFieldId) -> DatabaseResult<Vec<CustomFieldDataValue>> {
    let res = query("SELECT ID, UserId, FieldId, MemberId, DataValue, UpdatedAt FROM CustomFieldData WHERE UserId = ? AND FieldId = ?")
        .bind(user_id)
        .bind(field_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(res.into_iter().map(field_value).collect())
}

pub async fn get_field_values_for_member(pool: &DatabasePool, user_id: UserId, member_id: MemberId) -> DatabaseResult<Vec<CustomFieldDataValue>> {
    let res = query("SELECT ID, UserId, FieldId, MemberId, DataValue, UpdatedAt FROM CustomFieldData WHERE UserId = ? AND MemberId = ?")
        .bind(user_id)
        .bind(member_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(res.into_iter().map(field_value).collect())
}

pub async fn get_viewed_field_values_for_member(pool: &DatabasePool, user_id: UserId, member_id: MemberId, friend_viewer: Option<UserId>) -> DatabaseResult<Vec<ViewedCustomFieldDataValue>> {
    let res = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT cf.ID, Sort, Name, DataType, DataValue
FROM CustomField cf
INNER JOIN CustomFieldData cfd
ON cfd.FieldId = cf.ID
WHERE cf.UserId = ? AND cfd.MemberId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketCustomField pcf
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = pcf.BucketId AND pf.UserId = pcf.UserId
             WHERE pcf.FieldId = cf.ID AND pf.FriendId = ?
)
"#)
            .bind(user_id)
            .bind(member_id)
            .bind(friend_viewer)
            .fetch_all(pool.as_ref())
            .await?
    } else {
        query("SELECT cf.ID, Sort, Name, DataType, DataValue FROM CustomField cf INNER JOIN CustomFieldData cfd ON cfd.FieldId = cf.ID WHERE cf.UserId = ? AND cfd.MemberId = ?")
            .bind(user_id)
            .bind(member_id)
            .fetch_all(pool.as_ref())
            .await?
    };

    Ok(res.into_iter().map(|row| {
        let id = row.get("ID");
        let sort = row.get("Sort");
        let name = row.get("Name");
        let data_type = row.get("DataType");
        let data_type = CustomFieldDataType::from_repr(data_type).unwrap();
        let value = row.get("DataValue");
        ViewedCustomFieldDataValue {
            id,
            sort,
            name,
            data_type,
            value,
        }
    }).collect())
}

pub async fn get_field_value_by_id(pool: &DatabasePool, value_id: CustomFieldDataId, user_id: UserId) -> DatabaseResult<Option<CustomFieldDataValue>> {
    let res = query("SELECT ID, UserId, FieldId, MemberId, DataValue, UpdatedAt FROM CustomFieldData WHERE ID = ? AND UserId = ?")
        .bind(value_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(res.map(field_value))
}

pub async fn create_field_value(pool: &DatabasePool, value: &CustomFieldDataValue) -> DatabaseResult<CustomFieldDataId> {
    let id = query("INSERT INTO CustomFieldData (UserId, FieldId, MemberId, DataValue) VALUES (?, ?, ?, ?) RETURNING ID")
        .bind(value.user_id)
        .bind(value.field_id)
        .bind(value.member_id)
        .bind(&value.value)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn update_field_value(pool: &DatabasePool, value: &CustomFieldDataValue) -> DatabaseResult<()> {
    query("UPDATE CustomFieldData SET DataValue = ? WHERE ID = ? AND UserId = ?")
        .bind(&value.value)
        .bind(value.id)
        .bind(value.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn clear_field_value(pool: &DatabasePool, data_id: CustomFieldDataId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM CustomFieldData WHERE ID = ? AND UserId = ?")
        .bind(data_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_field_owner(pool: &DatabasePool, field_id: CustomFieldId) -> DatabaseResult<Option<UserId>> {
    let owner = query("SELECT UserId FROM CustomField WHERE ID = ?")
        .bind(field_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(owner.map(|row| row.get("UserId")))
}

fn field(row: MySqlRow) -> CustomField {
    let id = row.get("ID");
    let user_id = row.get("UserId");
    let sort = row.get("Sort");
    let name = row.get("Name");
    let data_type = row.get("DataType");
    let data_type = CustomFieldDataType::from_repr(data_type).unwrap();
    let updated_at = row.get("UpdatedAt");

    CustomField {
        id,
        user_id,
        sort,
        name,
        data_type,
        updated_at,
    }
}

fn field_value(row: MySqlRow) -> CustomFieldDataValue {
    let id = row.get("ID");
    let user_id = row.get("UserId");
    let field_id = row.get("FieldId");
    let member_id = row.get("MemberId");
    let value = row.get("DataValue");
    let updated_at = row.get("UpdatedAt");

    CustomFieldDataValue {
        id,
        user_id,
        field_id,
        member_id,
        value,
        updated_at,
    }
}