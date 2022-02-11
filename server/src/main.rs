#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;
use crate::config::PipeHubConfig;
use crate::data::Pool;
use crate::error::Result;
use crate::github::GitHubClient;
use crate::send::WeChatAccessToken;

use actix_cors::Cors;
use actix_session::CookieSession;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use dashmap::DashMap;
use diesel::{Connection, PgConnection};
use dotenv::dotenv;
use log::LevelFilter;
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use simplelog::{Config, TermLogger, TerminalMode};
use std::io;
use std::time::Duration;

mod config;
mod data;
mod error;
mod github;
mod models;
mod schema;
mod send;
mod user;
mod util;
mod wechat;

pub type AccessTokenCache = DashMap<i64, WeChatAccessToken>;

embed_migrations!("./migrations");

#[actix_rt::main]
async fn main() -> Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();

    dotenv().ok();

    let config = PipeHubConfig::new()?;
    migrate(&config);

    let https = config.https;
    let domain_web = config.domain_web.clone();
    let session_key: [u8; 32] = rand::random();

    let pool = web::Data::new(Pool::new(&config.database_url).await?);
    let github_client = web::Data::new(github_client(&config));
    let access_token_cache: web::Data<AccessTokenCache> = web::Data::new(DashMap::new());
    let http_client = web::Data::new(http_client());

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .app_data(github_client.clone())
            .app_data(access_token_cache.clone())
            .app_data(http_client.clone())
            .wrap(
                Cors::default()
                    .allowed_origin(&domain_web)
                    .allowed_methods(vec!["GET", "POST", "PUT"])
                    .allow_any_header()
                    .supports_credentials()
                    .expose_headers(vec!["Location"]),
            )
            .wrap(session(&session_key[..], https))
            .wrap(Compress::default())
            .wrap(Logger::default())
            .service(user::reset_key)
            .service(user::user)
            .service(user::update)
            .service(user::callback)
            .service(user::login)
            .service(wechat::wechat)
            .service(wechat::update)
            .service(
                web::resource("/send/{key}")
                    .route(web::get().to(send::send))
                    .route(web::post().to(send::send)),
            )
    })
    .bind(config.bind_addr())?
    .run()
    .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    error_message: String,
}

fn migrate(config: &PipeHubConfig) {
    let connection =
        PgConnection::establish(&config.database_url).expect("Unable to connect to DB.");

    embedded_migrations::run_with_output(&connection, &mut io::stdout())
        .expect("Unable to migrate.");
}

fn session(key: &[u8], https: bool) -> CookieSession {
    CookieSession::private(key)
        .name("session")
        .secure(https)
        .http_only(true)
}

fn github_client(config: &PipeHubConfig) -> GitHubClient {
    GitHubClient::new(
        config.github.client_id.clone(),
        config.github.client_secret.clone(),
        &config.github.callback_url,
    )
}

fn http_client() -> Client {
    ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create reqwest client.")
}
