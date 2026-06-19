use std::collections::HashMap;
use actix_web::{post, HttpRequest};
use actix_web::web::{Data, Json};
use crate::AppState;
use crate::database::to_web_error;
use crate::middleware::get_token;
use crate::model::fields::{CustomField, CustomFieldDataValue};
use crate::model::folder::Folder;
use crate::model::import::Import;
use crate::model::member::Member;
use crate::model::privacy::PrivacyBucket;
use crate::web::{ok_none, validation_error, WebResult};

#[post("/")]
pub async fn import(req: HttpRequest, data: Data<AppState>, body: Json<Import>) -> WebResult {
    let token = get_token(&req).unwrap();
    token.require_session()?;

    let body = body.into_inner();
    let mut transaction = data.pool.begin().await.map_err(|err| to_web_error(err.into()))?;
    {
        let privacy_mapping = if let Some(privacy) = body.privacy {
            let mut privacy_mapping = HashMap::new();
            for bucket in privacy {
                let import_id = bucket.id.clone();
                let mut actual_bucket: PrivacyBucket = bucket.into();
                actual_bucket.validate().map_err(validation_error)?;
                actual_bucket.user_id = token.user_id;
                let actual_id = crate::database::privacy::create_privacy_bucket(transaction.as_mut(), &actual_bucket).await.map_err(to_web_error)?;
                privacy_mapping.insert(import_id, actual_id);
            }
            Some(privacy_mapping)
        } else {
            None
        };

        let field_mapping = if let Some(fields) = body.fields {
            let mut field_mapping = HashMap::new();
            for field in fields {
                let import_id = field.id.clone();
                let privacy = field.privacy.clone();
                let mut actual_field: CustomField = field.into();
                actual_field.validate().map_err(validation_error)?;
                actual_field.user_id = token.user_id;
                let actual_id = crate::database::fields::create_field(transaction.as_mut(), &actual_field).await.map_err(to_web_error)?;
                field_mapping.insert(import_id, actual_id);

                if let Some(privacy_mapping) = &privacy_mapping {
                    for bucket in privacy {
                        if let Some(bucket_id) = privacy_mapping.get(&bucket) {
                            crate::database::privacy::add_privacy_bucket_custom_field(transaction.as_mut(), *bucket_id, token.user_id, actual_id).await.map_err(to_web_error)?;
                        }
                    }
                }
            }
            Some(field_mapping)
        } else {
            None
        };

        let folder_mapping = if let Some(folders) = body.folders {
            let mut folder_mapping = HashMap::new();
            for folder in folders {
                let import_id = folder.id.clone();
                let parent_id = folder.parent_id.clone();
                let privacy = folder.privacy.clone();
                let mut actual_folder: Folder = folder.into();
                actual_folder.validate().map_err(validation_error)?;
                actual_folder.user_id = token.user_id;
                let actual_id = crate::database::folder::create_folder(transaction.as_mut(), &actual_folder).await.map_err(to_web_error)?;
                folder_mapping.insert(import_id, (actual_id, parent_id));

                if let Some(privacy_mapping) = &privacy_mapping {
                    for bucket in privacy {
                        if let Some(bucket_id) = privacy_mapping.get(&bucket) {
                            crate::database::privacy::add_privacy_bucket_folder(transaction.as_mut(), *bucket_id, token.user_id, actual_id).await.map_err(to_web_error)?;
                        }
                    }
                }
            }
            for (_, (folder_id, parent_id)) in &folder_mapping {
                if let Some(parent_id) = parent_id {
                    if let Some((parent_id, _)) = folder_mapping.get(parent_id) {
                        crate::database::folder::change_parent(transaction.as_mut(), *folder_id, Some(*parent_id)).await.map_err(to_web_error)?;
                    }
                }
            }
            Some(folder_mapping)
        } else {
            None
        };

        if let Some(members) = body.members {
            for member in members {
                let privacy = member.privacy.clone();
                let fields = member.fields.clone();
                let folders = member.folders.clone();
                let mut actual_member: Member = member.into();
                actual_member.validate().map_err(validation_error)?;
                actual_member.user_id = token.user_id;
                let id = crate::database::member::create_member(transaction.as_mut(), &actual_member).await.map_err(to_web_error)?;

                if let Some(privacy_mapping) = &privacy_mapping {
                    for bucket in privacy {
                        if let Some(bucket_id) = privacy_mapping.get(&bucket) {
                            crate::database::privacy::add_privacy_bucket_member(transaction.as_mut(), *bucket_id, token.user_id, id).await.map_err(to_web_error)?;
                        }
                    }
                }

                if let Some(field_mapping) = &field_mapping {
                    for (field_id, field_value) in fields {
                        if let Some(field_id) = field_mapping.get(&field_id) {
                            let value = CustomFieldDataValue {
                                id: 0,
                                user_id: token.user_id,
                                field_id: *field_id,
                                member_id: id,
                                value: field_value,
                                updated_at: Default::default(),
                            };
                            crate::database::fields::create_field_value(transaction.as_mut(), &value).await.map_err(to_web_error)?;
                        }
                    }
                }

                if let Some(folder_mapping) = &folder_mapping {
                    for folder in folders {
                        if let Some((folder_id, _)) = folder_mapping.get(&folder) {
                            crate::database::member::add_member_folder(transaction.as_mut(), id, token.user_id, *folder_id).await.map_err(to_web_error)?;
                        }
                    }
                }
            }
        }
    }
    transaction.commit().await.map_err(|err| to_web_error(err.into()))?;

    ok_none()
}