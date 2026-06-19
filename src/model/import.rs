use crate::model::fields::{CustomField, CustomFieldDataType};
use crate::model::privacy::PrivacyBucket;
use serde::Deserialize;
use std::collections::HashMap;
use crate::model::folder::Folder;
use crate::model::member::Member;

#[derive(Deserialize)]
pub struct Import {
    pub privacy: Option<Vec<ImportPrivacyBucket>>,
    pub fields: Option<Vec<ImportCustomField>>,
    pub folders: Option<Vec<ImportFolder>>,
    pub members: Option<Vec<ImportMember>>,
}

#[derive(Deserialize)]
pub struct ImportPrivacyBucket {
    pub id: String,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub sort: u16,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
}

impl Into<PrivacyBucket> for ImportPrivacyBucket {
    fn into(self) -> PrivacyBucket {
        PrivacyBucket {
            sort: self.sort,
            name: self.name,
            description: self.description,
            emoji: self.emoji,
            color: self.color,
            user_id: 0,
            id: 0,
            folders: vec![],
            members: vec![],
            friends: vec![]
        }
    }
}

#[derive(Deserialize)]
pub struct ImportCustomField {
    pub id: String,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub sort: u16,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: CustomFieldDataType,
    pub privacy: Vec<String>,
}

impl Into<CustomField> for ImportCustomField {
    fn into(self) -> CustomField {
        CustomField {
            sort: self.sort,
            name: self.name,
            data_type: self.data_type,
            id: 0,
            user_id: 0,
            updated_at: Default::default()
        }
    }
}

#[derive(Deserialize)]
pub struct ImportFolder {
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
    pub privacy: Vec<String>,
}

impl Into<Folder> for ImportFolder {
    fn into(self) -> Folder {
        Folder {
            name: self.name,
            description: self.description,
            emoji: self.emoji,
            color: self.color,
            id: 0,
            user_id: 0,
            parent_id: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

#[derive(Deserialize)]
pub struct ImportMember {
    pub name: String,
    pub pronouns: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
    pub archived: bool,
    pub custom: bool,
    pub folders: Vec<String>,
    pub fields: HashMap<String, String>,
    pub privacy: Vec<String>,
}

impl Into<Member> for ImportMember {
    fn into(self) -> Member {
        Member {
            name: self.name,
            pronouns: self.pronouns,
            avatar: self.avatar,
            description: self.description,
            color: self.color,
            archived: self.archived,
            custom: self.custom,
            id: 0,
            user_id: 0,
            folders: vec![],
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}