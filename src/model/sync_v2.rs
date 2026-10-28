use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::fields::{CustomField, CustomFieldDataId, CustomFieldDataValue, CustomFieldId};
use crate::model::folder::{Folder, FolderId};
use crate::model::front::{FrontEntry, FrontEntryId};
use crate::model::gallery::{PhotoAlbum, PhotoAlbumId};
use crate::model::member::{Member, MemberId};
use crate::model::poll::{Poll, PollAnswer, PollAnswerId, PollId};
use crate::model::user::{UserId, UserInfo};

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub time: DateTime<Utc>,
    pub known: SyncIds,
    #[serde(default)]
    pub deleted: SyncIds,
    #[serde(default)]
    pub updated: SyncUpdates,
    pub front: Vec<FrontEntry>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub time: DateTime<Utc>,
    pub user: UserInfo,
    #[serde(rename = "friendCode")]
    pub friend_code: String,
    #[serde(skip_serializing_if = "is_empty")]
    pub delete: SyncIds,
    #[serde(skip_serializing_if = "is_empty")]
    pub assign: SyncIds,
    #[serde(skip_serializing_if = "is_empty")]
    pub update: SyncUpdates,
    #[serde(rename = "assignFront")]
    pub assign_front: Vec<FrontEntryId>,
    pub front: Vec<FrontEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SyncIds {
    pub folders: Vec<FolderId>,
    pub members: Vec<MemberId>,
    pub fields: Vec<CustomFieldId>,
    #[serde(rename = "fieldValues")]
    pub field_values: Vec<CustomFieldDataId>,
    pub polls: Vec<PollId>,
    #[serde(rename = "pollAnswers")]
    pub poll_answers: Vec<PollAnswerId>,
    #[serde(rename = "photoAlbums")]
    pub photo_albums: Vec<PhotoAlbumId>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SyncUpdates {
    pub folders: Vec<Folder>,
    pub members: Vec<Member>,
    pub fields: Vec<CustomField>,
    #[serde(rename = "fieldValues")]
    pub field_values: Vec<CustomFieldDataValue>,
    pub polls: Vec<Poll>,
    #[serde(rename = "pollAnswers")]
    pub poll_answers: Vec<PollAnswer>,
    #[serde(rename = "photoAlbums")]
    pub photo_albums: Vec<PhotoAlbum>,
}

#[derive(Debug, Deserialize)]
pub struct FrontSyncRequest {
    pub front: Vec<FrontEntry>,
}

#[derive(Debug, Serialize)]
pub struct FrontSyncResponse {
    #[serde(rename = "assignFront")]
    pub assign_front: Vec<FrontEntryId>,
    pub front: Vec<FrontEntry>,
}

pub trait Syncable {
    fn id(&self) -> i64;
    fn set_user(&mut self, user_id: UserId);
    fn updated_at(&self) -> &DateTime<Utc>;
}

trait MaybeEmpty {
    fn is_empty(&self) -> bool;
}

macro_rules! impl_maybe_empty {
    (for $($t:ty),+) => {
        $(impl MaybeEmpty for $t {
            fn is_empty(&self) -> bool {
                self.folders.is_empty() && self.members.is_empty() &&
                    self.fields.is_empty() && self.field_values.is_empty() &&
                    self.polls.is_empty() && self.poll_answers.is_empty() &&
                    self.photo_albums.is_empty()
            }
        })*
    };
}

impl_maybe_empty!(for SyncIds, SyncUpdates);

fn is_empty<T: MaybeEmpty>(v: &T) -> bool {
    v.is_empty()
}