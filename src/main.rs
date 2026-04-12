#![feature(fn_traits)]

mod database;
mod security;
mod web;
mod model;
mod error;
mod middleware;

use crate::database::DatabasePool;
use crate::web::auth::{login, register};
use crate::web::version::{app_update, version};
use actix_web::web::{scope, Data};
use actix_web::{App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::MySqlPool;
use std::env::var;
use std::sync::Arc;
use actix_web::dev::Service;
use crate::middleware::bearer_validation;
use crate::web::api::folder::{create_folder, delete_folder, edit_folder, get_folder, get_folders};
use crate::web::api::friend::{accept_friend_request, cancel_friend_request, decline_friend_request, get_friends, get_incoming_friend_requests, get_outgoing_friend_requests, get_settings, send_friend_request, unfriend, update_settings};
use crate::web::api::front::{add_front_entry, front, front_comment, front_start_time, unfront};
use crate::web::api::member::{create_member, delete_member, edit_member, edit_member_folders, get_member, get_members};
use crate::web::api::session::{get_sessions, invalidate_current_session, invalidate_session};
use crate::web::api::user::{get_front, get_self_user};

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

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(bearer_validation);

        App::new()
            .app_data(Data::new(AppState {
                pool: pool.clone(),
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
                scope("/api")
                    .wrap(auth)
                    .service(
                        scope("/folder")
                            .service(get_folders)
                            .service(get_folder)
                            .service(create_folder)
                            .service(delete_folder)
                            .service(edit_folder)
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
                    )
                    .service(
                        scope("/front")
                            .service(add_front_entry)
                            .service(front)
                            .service(unfront)
                            .service(front_comment)
                            .service(front_start_time)
                    )
                    .service(
                        scope("/member")
                            .service(get_members)
                            .service(get_member)
                            .service(create_member)
                            .service(delete_member)
                            .service(edit_member)
                            .service(edit_member_folders)
                    )
                    .service(
                        scope("/session")
                            .service(get_sessions)
                            .service(invalidate_session)
                            .service(invalidate_current_session)
                    )
                    .service(
                        scope("/user")
                            .service(get_self_user)
                            .service(get_front)
                    )
            )
            .service(
                scope("/auth")
                    .service(register)
                    .service(login)
            )
            .service(version)
            .service(app_update)
    }).bind(("0.0.0.0", 11675))?.run().await
}