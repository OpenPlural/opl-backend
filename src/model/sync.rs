use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::fields::{CustomField, CustomFieldDataId, CustomFieldDataValue, CustomFieldId};
use crate::model::folder::{Folder, FolderId};
use crate::model::front::FrontEntry;
use crate::model::member::{Member, MemberId};
use crate::model::user::UserInfo;

#[derive(Debug, Deserialize)]
pub struct SyncQuery {
    pub since: DateTime<Utc>,
    #[serde(default)]
    pub absolute: bool
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub time: DateTime<Utc>,
    pub user: UserInfo,
    #[serde(rename = "friendCode")]
    pub friend_code: String,
    #[serde(rename = "deletionDelta")]
    pub deletion_delta: bool,
    #[serde(rename = "folderIds")]
    pub folder_ids: Vec<FolderId>,
    #[serde(rename = "memberIds")]
    pub member_ids: Vec<MemberId>,
    #[serde(rename = "fieldIds")]
    pub field_ids: Vec<CustomFieldId>,
    #[serde(rename = "fieldValueIds")]
    pub field_value_ids: Vec<CustomFieldDataId>,
    #[serde(rename = "updatedFolders")]
    pub updated_folders: Vec<Folder>,
    #[serde(rename = "updatedMembers")]
    pub updated_members: Vec<Member>,
    #[serde(rename = "updatedFields")]
    pub updated_fields: Vec<CustomField>,
    #[serde(rename = "updatedFieldValues")]
    pub updated_field_values: Vec<CustomFieldDataValue>,
    pub front: Vec<FrontEntry>,
}