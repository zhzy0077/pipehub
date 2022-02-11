extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;
use crate::config::PipeHubConfig;
use crate::data::Pool;
use crate::error::{Error, Result};
use crate::github::GitHubClient;
use crate::send::WeChatAccessToken;
use actix_cors::Cors;
use actix_files::Files;
use actix_http::body::{Body, MessageBody, ResponseBody};
use actix_http::http::{header, Method, StatusCode, Uri};
use actix_http::HttpMessage;
use actix_session::CookieSession;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Compress, Logger};
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use actix_web::{Error as AWError, HttpResponse};
use dashmap::DashMap;
use diesel::{Connection, PgConnection};
use dotenv::dotenv;
use log::{info, Level};
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use std::future::Future;
use std::io;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use uuid::Uuid;

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
    openssl_probe::init_ssl_cert_env_vars();
    dotenv().ok();

    let config = PipeHubConfig::new()?;
    migrate(&config);

    let pool = Pool::new(&config.database_url).await?;
    let session_key: [u8; 32] = rand::random();
    let github_client = web::Data::new(github_client(&config));
    let https = config.https;
    let domain_web = config.domain_web.clone();
    let access_token_cache: Arc<AccessTokenCache> = Arc::new(DashMap::new());
    let http_client = http_client();

    let cloned_client = http_client.clone();
    tokio::spawn(async move {
        ping(cloned_client).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .app_data(github_client.clone())
            .data(access_token_cache.clone())
            .data(http_client.clone())
            .wrap(
                Cors::new()
                    .allowed_origin(&domain_web)
                    .allowed_methods(vec!["GET", "POST", "PUT"])
                    .supports_credentials()
                    .expose_headers(vec!["Location"])
                    .max_age(3600)
                    .finish(),
            )
            .wrap_fn(head_request)
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
                    .wrap(
                        Cors::new()
                            .send_wildcard()
                            .allowed_methods(vec!["GET", "POST"])
                            .finish(),
                    )
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

fn head_request<
    S: Service<Response = ServiceResponse<Body>, Request = ServiceRequest, Error = AWError>,
>(
    mut req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = std::result::Result<ServiceResponse<Body>, AWError>> {
    let is_head = req.method() == Method::HEAD;
    if is_head {
        req.head_mut().method = Method::GET;
    }
    let future = srv.call(req);
    async move {
        let res: std::result::Result<ServiceResponse<Body>, AWError> = future.await;
        if is_head {
            res.map(|res| res.map_body(|_, _| ResponseBody::Body(Body::Empty)))
        } else {
            res
        }
    }
}

fn http_client() -> Client {
    ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create reqwest client.")
}

async fn ping(client: Client) {
    let mut delay = time::interval(Duration::from_secs(30));
    loop {
        delay.tick().await;
        let resp = client
            .get("https://qyapi.weixin.qq.com/cgi-bin/gettoken")
            .send()
            .await;
        info!("Ping gettoken result {:?}.", resp)
    }
}
