use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;
use crate::model::member::MemberId;
use crate::model::user::UserId;
use crate::model::validate_string_length;

pub type CustomFieldId = i64;
pub type CustomFieldDataId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomField {
    #[serde(skip_deserializing)]
    pub id: CustomFieldId,
    #[serde(skip)]
    pub user_id: UserId,
    pub sort: u16,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: CustomFieldDataType,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRepr)]
#[repr(u8)]
pub enum CustomFieldDataType {
    Text = 0,
    Color = 1,
    Date = 2,
    Time = 3,
    DateTime = 4
}

impl CustomField {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("name", &self.name, Some(1), Some(255), false)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFieldDataValue {
    #[serde(skip_deserializing)]
    pub id: CustomFieldDataId,
    #[serde(skip)]
    pub user_id: UserId,
    #[serde(rename = "fieldId")]
    pub field_id: CustomFieldId,
    #[serde(rename = "memberId")]
    pub member_id: MemberId,
    pub value: String,
    #[serde(rename = "updatedAt", skip_deserializing)]
    pub updated_at: DateTime<Utc>,
}

impl CustomFieldDataValue {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length("value", &self.value, Some(1), Some(65535), false)?;
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
    pub value: String,
}