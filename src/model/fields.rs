use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;
use crate::model::member::MemberId;
use crate::model::sync_v2::Syncable;
use crate::model::user::UserId;
use crate::model::{validate_string_length, Validator};

pub type CustomFieldId = i64;
pub type CustomFieldDataId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomField {
    #[serde(default)]
    pub id: CustomFieldId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub sort: u16,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: CustomFieldDataType,
    pub config: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRepr)]
#[repr(u8)]
pub enum CustomFieldDataType {
    Text = 0,
    Color = 1,
    Date = 2,
    Time = 3,
    DateTime = 4,
    DayMonth = 5,
    NamedScale = 6,
}

impl Syncable for CustomField {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_user(&mut self, user_id: UserId) {
        self.user_id = user_id;
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl Validator for CustomField {
    fn validate(&self) -> Result<(), String> {
        validate_string_length("CustomField", "name", &self.name, Some(1), Some(255), false)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFieldDataValue {
    #[serde(default)]
    pub id: CustomFieldDataId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "fieldId", deserialize_with = "crate::numberstring::deserialize")]
    pub field_id: CustomFieldId,
    #[serde(rename = "memberId", deserialize_with = "crate::numberstring::deserialize")]
    pub member_id: MemberId,
    pub value: String,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: DateTime<Utc>,
}

impl Syncable for CustomFieldDataValue {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_user(&mut self, user_id: UserId) {
        self.user_id = user_id;
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl Validator for CustomFieldDataValue {
    fn validate(&self) -> Result<(), String> {
        validate_string_length("CustomFieldDataValue", "value", &self.value, Some(1), Some(65535), false)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ViewedCustomFieldDataValue {
    pub id: CustomFieldId,
    pub sort: u16,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: CustomFieldDataType,
    pub config: Option<String>,
    pub value: String,
}