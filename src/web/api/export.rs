use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::folder::FolderId;
use crate::model::import::{Import, ImportCustomField, ImportFolder, ImportMember, ImportPrivacyBucket};
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

    let privacy = crate::database::privacy::get_privacy_buckets(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let privacy = privacy.into_iter().map(|pb| ImportPrivacyBucket {
        id: pb.id.to_string(),
        sort: pb.sort,
        name: pb.name,
        description: pb.description,
        emoji: pb.emoji,
        color: pb.color,
    }).collect();

    let folder_privacy = crate::database::privacy::get_folder_privacy_entries(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let mut folder_privacy: HashMap<FolderId, Vec<String>> = list_to_map(folder_privacy);
    let folders = crate::database::folder::get_folders(&data.pool, token.user_id, None).await.map_err(to_web_error)?;
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

    let custom_field_privacy = crate::database::privacy::get_custom_field_privacy_entries(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let mut custom_field_privacy = list_to_map(custom_field_privacy);
    let custom_fields = crate::database::fields::get_fields(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let custom_fields = custom_fields.into_iter().map(|f| ImportCustomField {
        id: f.id.to_string(),
        sort: f.sort,
        name: f.name.to_string(),
        data_type: f.data_type,
        privacy: custom_field_privacy.remove(&f.id).unwrap_or_default(),
    }).collect();

    let custom_field_data = crate::database::fields::get_field_values(&data.pool, token.user_id).await.map_err(to_web_error)?;
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

    let member_privacy = crate::database::privacy::get_member_privacy_entries(&data.pool, token.user_id).await.map_err(to_web_error)?;
    let mut member_privacy = list_to_map(member_privacy);
    let members = crate::database::member::get_members(&data.pool, token.user_id, None).await.map_err(to_web_error)?;
    let members = members.into_iter().map(|m| ImportMember {
        name: m.id.to_string(),
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

    ok(Import {
        privacy: Some(privacy),
        fields: Some(custom_fields),
        folders: Some(folders),
        members: Some(members),
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