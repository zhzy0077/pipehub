#[macro_use]
extern crate lazy_static;
use crate::config::PipeHubConfig;
use crate::data::Pool;
use crate::error::Result;
use crate::github::GitHubClient;
use crate::request_id::{RequestId, RequestIdAware};
use crate::send::WeChatAccessToken;

use crate::microsoft::MicrosoftClient;
use actix_cors::Cors;
use actix_files::Files;
use actix_http::HttpMessage;
use actix_session::CookieSession;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dashmap::DashMap;
use dotenv::dotenv;
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use std::time::Duration;

mod captcha;
mod config;
mod data;
mod error;
mod github;
mod microsoft;
mod models;
mod request_id;
mod send;
mod user;
mod util;
mod wechat;

pub type AccessTokenCache = DashMap<i64, WeChatAccessToken>;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let config = PipeHubConfig::new()?;

    let https = config.https;
    let session_key: [u8; 32] = rand::random();

    let pool = Pool::new(&config.database_url).await?;
    pool.migrate().await?;

    let pool = web::Data::new(pool);
    let github_client = web::Data::new(github_client(&config));
    let microsoft_client = web::Data::new(microsoft_client(&config));
    let access_token_cache: web::Data<AccessTokenCache> = web::Data::new(DashMap::new());
    let http_client = web::Data::new(http_client());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allow_any_header()
            .supports_credentials()
            .expose_headers(vec!["Location"]);

        let logger = Logger::new(r#"%t %{request_id}xi "%r" %s %b %T"#)
            .custom_request_replace("request_id", |req| {
                req.extensions().get::<RequestId>().unwrap().to_string()
            });

        App::new()
            .app_data(pool.clone())
            .app_data(github_client.clone())
            .app_data(microsoft_client.clone())
            .app_data(access_token_cache.clone())
            .app_data(http_client.clone())
            .wrap(cors)
            .wrap(session(&session_key[..], https))
            .wrap(logger)
            .wrap(RequestIdAware)
            .service(user::reset_key)
            .service(user::user)
            .service(user::update)
            .service(user::callback)
            .service(user::login)
            .service(user::msft_auth_url)
            .service(user::msft_callback)
            .service(wechat::wechat)
            .service(wechat::update)
            .service(
                web::resource("/send/{key}")
                    .route(web::get().to(send::send))
                    .route(web::post().to(send::send)),
            )
            .service(Files::new("/", "./static/").use_hidden_files())
    })
    .workers(num_cpus::get())
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

fn microsoft_client(config: &PipeHubConfig) -> MicrosoftClient {
    MicrosoftClient::new(
        config.microsoft.client_id.clone(),
        config.microsoft.client_secret.clone(),
        &config.microsoft.callback_url,
    )
}

fn http_client() -> Client {
    ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(8)
        .build()
        .expect("Failed to create reqwest client.")
}
