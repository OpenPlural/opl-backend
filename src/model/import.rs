use crate::model::fields::{CustomField, CustomFieldDataType};
use crate::model::privacy::PrivacyBucket;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::model::folder::Folder;
use crate::model::member::Member;
use crate::model::poll::{Poll, POLL_MAX_OPTIONS};

#[derive(Deserialize, Serialize)]
pub struct Import {
    pub privacy: Option<Vec<ImportPrivacyBucket>>,
    pub fields: Option<Vec<ImportCustomField>>,
    pub folders: Option<Vec<ImportFolder>>,
    pub members: Option<Vec<ImportMember>>,
    pub polls: Option<Vec<ImportPoll>>,
    #[serde(skip_serializing)]
    pub truncate: bool,
}

#[derive(Serialize, Deserialize)]
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

impl ImportPrivacyBucket {
    pub fn truncate(&mut self) {
        self.name.truncate(self.name.floor_char_boundary(255));
        if let Some(description) = &mut self.description {
            description.truncate(description.floor_char_boundary(65535));
        }
        if let Some(emoji) = &mut self.emoji {
            emoji.truncate(emoji.floor_char_boundary(30));
        }
    }
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

#[derive(Serialize, Deserialize)]
pub struct ImportCustomField {
    pub id: String,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub sort: u16,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: CustomFieldDataType,
    pub privacy: Vec<String>,
}

impl ImportCustomField {
    pub fn truncate(&mut self) {
        self.name.truncate(self.name.floor_char_boundary(255));
    }
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

#[derive(Serialize, Deserialize)]
pub struct ImportFolder {
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
    pub sort: u16,
    pub privacy: Vec<String>,
}

impl ImportFolder {
    pub fn truncate(&mut self) {
        self.name.truncate(self.name.floor_char_boundary(255));
        if let Some(description) = &mut self.description {
            description.truncate(description.floor_char_boundary(65535));
        }
        if let Some(emoji) = &mut self.emoji {
            emoji.truncate(emoji.floor_char_boundary(30));
        }
    }
}

impl Into<Folder> for ImportFolder {
    fn into(self) -> Folder {
        Folder {
            name: self.name,
            description: self.description,
            emoji: self.emoji,
            color: self.color,
            sort: self.sort,
            id: 0,
            user_id: 0,
            parent_id: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ImportMember {
    pub id: String,
    pub name: String,
    pub pronouns: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "crate::numberstring::deserialize")]
    pub color: u32,
    pub archived: bool,
    pub custom: bool,
    pub sort: u16,
    pub folders: Vec<String>,
    pub fields: HashMap<String, String>,
    pub privacy: Vec<String>,
}

impl ImportMember {
    pub fn truncate(&mut self) {
        self.name.truncate(self.name.floor_char_boundary(255));
        if let Some(pronouns) = &mut self.pronouns {
            pronouns.truncate(pronouns.floor_char_boundary(255));
        }
        if let Some(avatar) = &mut self.avatar {
            avatar.truncate(avatar.floor_char_boundary(255));
        }
        if let Some(description) = &mut self.description {
            description.truncate(description.floor_char_boundary(65535));
        }
    }
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
            sort: self.sort,
            id: 0,
            user_id: 0,
            folders: vec![],
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ImportPoll {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "allowAbstain")]
    pub allow_abstain: bool,
    #[serde(rename = "allowVeto")]
    pub allow_veto: bool,
    #[serde(rename = "openUntil")]
    pub open_until: DateTime<Utc>,
    #[serde(rename = "customOptions")]
    pub custom_options: Option<Vec<String>>,
    pub answers: Vec<ImportPollAnswer>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImportPollAnswer {
    #[serde(rename = "memberId")]
    pub member_id: String,
    pub answer: u8,
    pub comment: Option<String>,
}

impl ImportPoll {
    pub fn truncate(&mut self) {
        self.name.truncate(self.name.floor_char_boundary(255));
        if let Some(description) = &mut self.description {
            description.truncate(description.floor_char_boundary(65535));
        }
        if let Some(custom_options) = &mut self.custom_options {
            custom_options.truncate(POLL_MAX_OPTIONS);
        }
        for answer in &mut self.answers {
            if let Some(comment) = &mut answer.comment {
                comment.truncate(comment.floor_char_boundary(255));
            }
        }
    }
}

impl Into<Poll> for ImportPoll {
    fn into(self) -> Poll {
        Poll {
            name: self.name,
            description: self.description,
            id: 0,
            user_id: 0,
            allow_abstain: self.allow_abstain,
            allow_veto: self.allow_veto,
            open_until: self.open_until,
            custom_options: self.custom_options,
            updated_at: Default::default(),
        }
    }
}