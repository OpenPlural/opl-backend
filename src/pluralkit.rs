use crate::database::{to_web_error, DatabasePool};
use crate::error::WebError;
use crate::model::member::Member;
use pluralkit::PKClient;
use tokio::sync::OnceCell;
use crate::model::user::UserId;

static CLIENT: OnceCell<PKClient> = OnceCell::const_new();

async fn get_client() -> &'static PKClient {
    CLIENT.get_or_init(|| async {
        let mut client = PKClient::new("Open Plural Server".to_string());
        client.max_retries = 5;
        client
    }).await
}

pub async fn pull(pool: &DatabasePool, token: &str, user_id: UserId, members: Vec<Member>) -> Result<(), WebError> {
    let client = get_client().await;
    let pk_members = client.get_system_members(Some(token), "@me").await.map_err(|e| WebError::PluralKitError(e))?;
    let mut transaction = pool.begin().await.map_err(|err| to_web_error(err.into()))?;
    for pk_member in pk_members {
        let member = members.iter().find(|m| m.pk_id.as_ref().map(|id| pk_member.id.eq(id)).unwrap_or_default());
        if let Some(member) = member {
            // Update
            let mut member = member.clone();
            let mut update = false;

            let pk_name = pk_member.name.unwrap();
            if member.name != pk_name {
                member.name = pk_name;
                update = true;
            }

            let pk_color = pk_member.color.map(|c| convert_hex_code_to_color(&c)).unwrap_or(16777215);
            if member.color != pk_color {
                member.color = pk_color;
                update = true;
            }

            update_if_different(&mut member.pronouns, &pk_member.pronouns, &mut update);
            update_if_different(&mut member.description, &pk_member.description, &mut update);
            if member.avatar.is_none() {
                update_if_different(&mut member.avatar, &pk_member.avatar_url, &mut update);
            }

            if update {
                crate::database::member::edit_member(transaction.as_mut(), &member).await.map_err(to_web_error)?;
            }
        } else {
            // Create
            let member = Member {
                id: 0,
                user_id,
                pk_id: Some(pk_member.id),
                sort: 0,
                name: pk_member.name.unwrap(),
                pronouns: pk_member.pronouns,
                avatar: pk_member.avatar_url,
                description: pk_member.description,
                color: pk_member.color.map(|c| convert_hex_code_to_color(&c)).unwrap_or(16777215),
                archived: false,
                custom: false,
                created_at: Default::default(),
                updated_at: Default::default(),
                folders: vec![],
            };
            crate::database::member::create_member(transaction.as_mut(), &member).await.map_err(to_web_error)?;
        }
    }
    transaction.commit().await.map_err(|err| to_web_error(err.into()))?;
    Ok(())
}

pub async fn push(pool: &DatabasePool, token: &str, display_name: Option<String>, members: Vec<Member>) -> Result<(), WebError> {
    let client = get_client().await;
    let pk_members = client.get_system_members(Some(token), "@me").await.map_err(|e| WebError::PluralKitError(e))?;
    let mut newly_assigned = Vec::with_capacity(pk_members.len());
    for member in &members {
        let pk_id = if let Some(pk_id) = &member.pk_id {
            Some(pk_id.clone())
        } else {
            let new_id = pk_members.iter().find(|pm| {
                if newly_assigned.contains(&pm.id) {
                    return false;
                }
                if let Some(name) = &pm.name {
                    if member.name.eq(name) {
                        return members.iter().find(|m| m.pk_id.as_ref().map(|id| pm.id.eq(id)).unwrap_or_default()).is_none();
                    }
                }
                false
            }).map(|m| m.id.clone());
            if let Some(new_id) = &new_id {
                newly_assigned.push(new_id.clone());
                crate::database::pluralkit::assign_member_pk_id(pool, member.user_id, member.id, new_id).await.map_err(to_web_error)?;
            }
            new_id
        };

        if let Some(pk_id) = pk_id {
            // Update
            if let Some(pk_member) = pk_members.iter().find(|pm| pm.id.eq(&pk_id)) {
                let mut pk_member = pk_member.clone();
                let mut update = false;
                update_if_different(&mut pk_member.name, &Some(member.name.clone()), &mut update);
                update_if_different(&mut pk_member.pronouns, &member.pronouns, &mut update);
                update_if_different(&mut pk_member.description, &member.description, &mut update);
                update_if_different(&mut pk_member.color, &Some(convert_color_to_hex_code(member.color)), &mut update);
                update_if_different(&mut pk_member.avatar_url, &member.avatar.as_ref().filter(|u| !u.starts_with(":cdn:")).cloned(), &mut update);
                if let Some(display_name) = &display_name {
                    update_if_different(&mut pk_member.display_name, &Some(format_display_name(display_name, &member)), &mut update);
                }
                if update {
                    client.update_member(token, &pk_member).await.map_err(|e| WebError::PluralKitError(e))?;
                }
                continue;
            }
        }

        // Create
        let pk_member = pluralkit::model::members::Member {
            name: Some(member.name.clone()),
            pronouns: member.pronouns.clone(),
            description: member.description.clone(),
            avatar_url: member.avatar.as_ref().filter(|u| !u.starts_with(":cdn:")).cloned(),
            color: Some(convert_color_to_hex_code(member.color)),
            display_name: display_name.as_ref().map(|t| format_display_name(t, &member)),
            ..Default::default()
        };
        let pk_member = client.create_member(token, &pk_member).await.map_err(|e| WebError::PluralKitError(e))?;
        crate::database::pluralkit::assign_member_pk_id(&pool, member.user_id, member.id, &pk_member.id).await.map_err(to_web_error)?;
    }
    Ok(())
}

fn update_if_different(old_value: &mut Option<String>, new_value: &Option<String>, update: &mut bool) {
    if let Some(old_value) = old_value {
        if let Some(new_value) = new_value {
            if old_value != new_value {
                *old_value = new_value.clone();
                *update = true;
            }
        } else {
            *old_value = "".to_string();
            *update = true;
        }
    } else if new_value.is_some() {
        *old_value = new_value.clone();
        *update = true;
    }
}

fn format_display_name(template: &str, member: &Member) -> String {
    template.replace("%name%", &member.name)
        .replace("%pronouns%", member.pronouns.as_ref().map(|s| s.as_str()).unwrap_or_default())
}

fn convert_color_to_hex_code(color: u32) -> String {
    let r = (color >> 16) & 0xff;
    let g = (color >> 8) & 0xff;
    let b = color & 0xff;
    format!("{:02x}{:02x}{:02x}", r, g, b)
}

fn convert_hex_code_to_color(color: &str) -> u32 {
    if color.len() == 6 {
        let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(255) as u32;
        let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(255) as u32;
        let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(255) as u32;
        return (r << 16) | (g << 8) | b;
    }
    16777215
}