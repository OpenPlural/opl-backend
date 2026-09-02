use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::folder::FolderId;
use crate::model::import::{Import, ImportCustomField, ImportFolder, ImportMember, ImportPhotoAlbum, ImportPoll, ImportPollAnswer, ImportPrivacyBucket};
use crate::model::member::MemberId;
use crate::web::{ok, WebResult};
use crate::AppState;
use actix_web::web::Data;
use actix_web::{post, HttpRequest};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OnceCell};
use tokio::time::Instant;
use crate::error::WebError;
use crate::list_map::append;
use crate::model::poll::PollId;
use crate::model::user::UserId;

const COOLDOWN_DURATION: Duration = Duration::from_hours(6);
static USER_COOLDOWN: OnceCell<Arc<Mutex<HashMap<UserId, Instant>>>> = OnceCell::const_new();

async fn get_user_cooldowns() -> &'static Arc<Mutex<HashMap<UserId, Instant>>> {
    USER_COOLDOWN.get_or_init(|| async {
        Arc::new(Mutex::new(HashMap::new()))
    }).await
}

#[post("/")]
pub async fn export(req: HttpRequest, data: Data<AppState>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_session()?;

    {
        let user_cooldown = get_user_cooldowns().await;
        let mut user_cooldown = user_cooldown.lock().await;
        user_cooldown.retain(|_, v| v.elapsed() <= COOLDOWN_DURATION);

        if user_cooldown.contains_key(&token.user_id) {
            return Err(WebError::WaitCooldown("6 hours"));
        }

        user_cooldown.insert(token.user_id, Instant::now());
    }

    do_export(data, token.user_id).await
}

pub async fn do_export(data: Data<AppState>, user_id: UserId) -> WebResult {
    let privacy = crate::database::privacy::get_privacy_buckets(&data.pool, user_id).await.map_err(to_web_error)?;
    let privacy = privacy.into_iter().map(|pb| ImportPrivacyBucket {
        id: pb.id.to_string(),
        sort: pb.sort,
        name: pb.name,
        description: pb.description,
        emoji: pb.emoji,
        color: pb.color,
    }).collect();

    let folder_privacy = crate::database::privacy::get_folder_privacy_entries(&data.pool, user_id).await.map_err(to_web_error)?;
    let mut folder_privacy: HashMap<FolderId, Vec<String>> = list_to_map(folder_privacy);
    let folders = crate::database::folder::get_folders(&data.pool, user_id, None).await.map_err(to_web_error)?;
    let folders = folders.into_iter().map(|f| ImportFolder {
        id: f.id.to_string(),
        parent_id: f.parent_id.map(|id| id.to_string()),
        name: f.name.to_string(),
        description: f.description,
        emoji: f.emoji,
        color: f.color,
        sort: f.sort,
        privacy: folder_privacy.remove(&f.id).unwrap_or_default(),
    }).collect();

    let custom_field_privacy = crate::database::privacy::get_custom_field_privacy_entries(&data.pool, user_id).await.map_err(to_web_error)?;
    let mut custom_field_privacy = list_to_map(custom_field_privacy);
    let custom_fields = crate::database::fields::get_fields(&data.pool, user_id).await.map_err(to_web_error)?;
    let custom_fields = custom_fields.into_iter().map(|f| ImportCustomField {
        id: f.id.to_string(),
        sort: f.sort,
        name: f.name.to_string(),
        data_type: f.data_type,
        privacy: custom_field_privacy.remove(&f.id).unwrap_or_default(),
    }).collect();

    let custom_field_data = crate::database::fields::get_field_values(&data.pool, user_id).await.map_err(to_web_error)?;
    let mut custom_field_data: HashMap<MemberId, HashMap<String, String>> = custom_field_data.into_iter().fold(HashMap::new(), |mut map, field| {
        if let Some(map) = map.get_mut(&field.member_id) {
            map.insert(field.field_id.to_string(), field.value);
        } else {
            let mut field_map = HashMap::new();
            field_map.insert(field.field_id.to_string(), field.value);
            map.insert(field.member_id, field_map);
        }
        map
    });

    let member_privacy = crate::database::privacy::get_member_privacy_entries(&data.pool, user_id).await.map_err(to_web_error)?;
    let mut member_privacy = list_to_map(member_privacy);
    let members = crate::database::member::get_members(&data.pool, user_id, None).await.map_err(to_web_error)?;
    let members = members.into_iter().map(|m| ImportMember {
        id: m.id.to_string(),
        name: m.name.to_string(),
        pronouns: m.pronouns,
        avatar: m.avatar,
        description: m.description,
        color: m.color,
        archived: m.archived,
        custom: m.custom,
        sort: m.sort,
        folders: m.folders.into_iter().map(|id| id.to_string()).collect(),
        fields: custom_field_data.remove(&m.id).unwrap_or_default(),
        privacy: member_privacy.remove(&m.id).unwrap_or_default(),
    }).collect();

    let poll_answers = crate::database::poll::get_poll_answers_by_user_id(&data.pool, user_id).await.map_err(to_web_error)?;
    let mut poll_answers: HashMap<PollId, Vec<ImportPollAnswer>> = poll_answers.into_iter().fold(HashMap::new(), |mut map, answer| {
        let poll_id = answer.poll_id;
        let answer = ImportPollAnswer {
            member_id: answer.member_id.to_string(),
            answer: answer.answer,
            comment: answer.comment,
        };
        append(&mut map, poll_id, answer);
        map
    });

    let polls = crate::database::poll::get_polls(&data.pool, user_id).await.map_err(to_web_error)?;
    let polls = polls.into_iter().map(|p| ImportPoll {
        name: p.name.clone(),
        description: p.description.clone(),
        allow_abstain: p.allow_abstain,
        allow_veto: p.allow_veto,
        open_until: p.open_until,
        custom_options: p.custom_options.clone(),
        answers: poll_answers.remove(&p.id).unwrap_or_default(),
    }).collect();

    let gallery = crate::database::gallery::get_photo_albums(&data.pool, user_id).await.map_err(to_web_error)?;
    let gallery = gallery.into_iter().map(|a| ImportPhotoAlbum {
        member_id: a.member_id.to_string(),
        sort: a.sort,
        name: a.name,
        description: a.description,
        photo_urls: a.photo_urls,
    }).collect();

    ok(Import {
        privacy: Some(privacy),
        fields: Some(custom_fields),
        folders: Some(folders),
        members: Some(members),
        polls: Some(polls),
        gallery: Some(gallery),
        truncate: false,
    })
}

fn list_to_map<K: Eq + Hash, V: Sized + ToString>(list: Vec<(K, V)>) -> HashMap<K, Vec<String>> {
    list.into_iter().fold(HashMap::new(), |mut map, (key, value)| {
        if let Some(list) = map.get_mut(&key) {
            list.push(value.to_string());
        } else {
            map.insert(key, vec![value.to_string()]);
        }
        map
    })
}