#![feature(fn_traits)]
#![feature(trait_alias)]

mod database;
mod security;
mod web;
mod model;
mod error;
mod middleware;
mod notification;
mod frontwatch;
mod numberstring;

use crate::database::DatabasePool;
use crate::middleware::authenticator_mw;
use crate::web::api::folder::{create_folder, delete_folder, edit_folder, get_folder, get_folder_privacy, get_folders, set_folder_privacy};
use crate::web::api::friend::{accept_friend_request, cancel_friend_request, decline_friend_request, get_friend_privacy, get_friends, get_incoming_friend_requests, get_outgoing_friend_requests, get_settings, send_friend_request, unfriend, update_settings};
use crate::web::api::front::{add_front_entry, delete_front_entry, edit_front_entry, get_front_entries, get_front_entry, get_front_history};
use crate::web::api::member::{create_member, delete_member, edit_member, edit_member_folders, get_member, get_member_field_values, get_member_fields, get_member_front_entry, get_member_front_history, get_member_privacy, get_members};
use crate::web::api::session::{get_sessions, invalidate_current_session, invalidate_session};
use crate::web::api::sync::sync;
use crate::web::api::user::{change_friend_code, edit_user, get_self_user, get_user, get_username};
use crate::web::auth::{change_password, delete_account, login, register, reset_password};
use crate::web::version::version;
use actix_web::dev::Service;
use actix_web::web::{scope, Data};
use actix_web::{App, HttpServer};
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{migrate, MySqlPool};
use std::env::var;
use std::sync::Arc;
use std::time::Duration;
use actix_web::middleware::from_fn;
use tokio::spawn;
use tokio::time::interval;
use crate::frontwatch::watch_front_changes;
use crate::web::admin::make_password_reset_token;
use crate::web::api::apikey::{create_api_key, delete_api_key, get_api_keys};
use crate::web::api::fields::{clear_field_value, create_field, create_field_value, delete_field, edit_field, get_field, get_field_privacy, get_field_value, get_field_values, get_fields, get_specific_field_values, reorder_fields, update_field_value};
use crate::web::api::import::import;
use crate::web::api::notification::subscribe;
use crate::web::api::privacy::{add_privacy_bucket_custom_field, add_privacy_bucket_folder, add_privacy_bucket_friend, add_privacy_bucket_member, create_privacy_bucket, delete_privacy_bucket, edit_privacy_bucket, get_privacy_bucket, get_privacy_buckets, remove_privacy_bucket_custom_field, remove_privacy_bucket_folder, remove_privacy_bucket_friend, remove_privacy_bucket_member, reorder_privacy_buckets};

#[derive(Clone)]
pub struct AppState {
    pub pool: DatabasePool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = Arc::new(
        MySqlPool::connect_with(MySqlConnectOptions::new()
            .host(var("DB_HOST").expect("DB_HOST not set").as_str())
            .port(var("DB_PORT").expect("DB_PORT not set").parse().expect("DB_PORT not parseable"))
            .database(var("DB_NAME").expect("DB_NAME not set").as_str())
            .username(var("DB_USER").expect("DB_USER not set").as_str())
            .password(var("DB_PASS").expect("DB_PASS not set").as_str())
        ).await.expect("Failed to connect to database"));

    migrate!()
        .run(&*pool)
        .await
        .expect("Failed to run database migrations");

    let db_pool = pool.clone();
    watch_front_changes(db_pool.clone()).await;
    spawn(async move {
        let mut interval = interval(Duration::from_hours(6));
        loop {
            interval.tick().await;
            if let Err(err) = database::session::clear_expired_sessions(&db_pool).await {
                eprintln!("Failed to clear expired sessions: {:?}", err);
            }
            if let Err(err) = database::deletion::clear_old_deletions(&db_pool).await {
                eprintln!("Failed to clear old deletions: {:?}", err);
            }
            if let Err(err) = database::user::clear_expired_password_reset_tokens(&db_pool).await {
                eprintln!("Failed to clear expired password reset tokens: {:?}", err);
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                pool: pool.clone()
            }))
            .wrap_fn(|req, route| {
                println!("{} {}", req.method(), req.path());

                let res = route.call(req);
                async {
                    let res = res.await?;
                    Ok(res)
                }
            })
            .service(
                scope("/api/v1")
                    .wrap(from_fn(authenticator_mw))
                    .service(
                        scope("/api-key")
                            .service(get_api_keys)
                            .service(create_api_key)
                            .service(delete_api_key)
                    )
                    .service(
                        scope("/field")
                            .service(get_fields)
                            .service(get_field)
                            .service(create_field)
                            .service(delete_field)
                            .service(edit_field)
                            .service(reorder_fields)
                            .service(get_specific_field_values)
                            .service(get_field_values)
                            .service(get_field_value)
                            .service(create_field_value)
                            .service(update_field_value)
                            .service(clear_field_value)
                            .service(get_field_privacy)
                    )
                    .service(
                        scope("/folder")
                            .service(get_folders)
                            .service(get_folder)
                            .service(create_folder)
                            .service(delete_folder)
                            .service(edit_folder)
                            .service(get_folder_privacy)
                            .service(set_folder_privacy)
                    )
                    .service(
                        scope("/friend")
                            .service(get_friends)
                            .service(get_incoming_friend_requests)
                            .service(get_outgoing_friend_requests)
                            .service(send_friend_request)
                            .service(cancel_friend_request)
                            .service(accept_friend_request)
                            .service(decline_friend_request)
                            .service(unfriend)
                            .service(get_settings)
                            .service(update_settings)
                            .service(get_friend_privacy)
                    )
                    .service(
                        scope("/front")
                            .service(get_front_history)
                            .service(get_front_entries)
                            .service(get_front_entry)
                            .service(add_front_entry)
                            .service(delete_front_entry)
                            .service(edit_front_entry)
                    )
                    .service(
                        scope("/import")
                            .service(import)
                    )
                    .service(
                        scope("/member")
                            .service(get_members)
                            .service(get_member)
                            .service(create_member)
                            .service(delete_member)
                            .service(edit_member)
                            .service(edit_member_folders)
                            .service(get_member_field_values)
                            .service(get_member_fields)
                            .service(get_member_front_history)
                            .service(get_member_front_entry)
                            .service(get_member_privacy)
                    )
                    .service(
                        scope("/notification")
                            .service(subscribe)
                    )
                    .service(
                        scope("/privacy")
                            .service(get_privacy_buckets)
                            .service(get_privacy_bucket)
                            .service(create_privacy_bucket)
                            .service(delete_privacy_bucket)
                            .service(edit_privacy_bucket)
                            .service(reorder_privacy_buckets)
                            .service(add_privacy_bucket_folder)
                            .service(add_privacy_bucket_member)
                            .service(add_privacy_bucket_custom_field)
                            .service(add_privacy_bucket_friend)
                            .service(remove_privacy_bucket_folder)
                            .service(remove_privacy_bucket_member)
                            .service(remove_privacy_bucket_custom_field)
                            .service(remove_privacy_bucket_friend)
                    )
                    .service(
                        scope("/session")
                            .service(get_sessions)
                            .service(invalidate_current_session)
                            .service(invalidate_session)
                    )
                    .service(
                        scope("/sync")
                            .service(sync)
                    )
                    .service(
                        scope("/user")
                            .service(get_self_user)
                            .service(get_user)
                            .service(get_username)
                            .service(edit_user)
                            .service(change_friend_code)
                    )
            )
            .service(
                scope("/auth")
                    .service(register)
                    .service(login)
                    .service(delete_account)
                    .service(change_password)
                    .service(reset_password)
            )
            .service(
                scope("/admin")
                    .service(make_password_reset_token)
            )
            .service(version)
    }).bind(("0.0.0.0", 11675))?.run().await
}