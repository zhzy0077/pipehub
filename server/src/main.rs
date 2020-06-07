extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::config::PipeHubConfig;
use crate::error::{Error, Result};
use crate::logger::ApplicationLogger;
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
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use log::{info, Level, LevelFilter};
use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use r2d2::PooledConnection;
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use simplelog::{Config, TermLogger, TerminalMode};
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
mod logger;
mod models;
mod schema;
mod send;
mod user;
mod util;
mod wechat;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;
pub type AccessTokenCache = DashMap<i64, WeChatAccessToken>;
const HINT: &'static str =
    "If you believe it's unexpected. Please help us by creating an issue with this response at https://github.com/zhzy0077/pipehub.";

embed_migrations!("./migrations");

#[actix_rt::main]
async fn main() -> Result<()> {
    openssl_probe::init_ssl_cert_env_vars();
    dotenv().ok();

    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap();

    let config = PipeHubConfig::new()?;

    migrate(&config);

    let logger = Arc::new(ApplicationLogger::new(&config.log).await);

    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool: DbPool = Pool::new(manager)?;

    let session_key: [u8; 32] = rand::random();
    let github_client = Arc::new(client(&config));
    let https = config.https;
    let access_token_cache: Arc<AccessTokenCache> = Arc::new(DashMap::new());
    let http_client = http_client();

    let cloned_client = http_client.clone();
    tokio::spawn(async move {
        ping(cloned_client).await;
    });

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(github_client.clone())
            .data(logger.clone())
            .data(access_token_cache.clone())
            .data(http_client.clone())
            .wrap_fn(head_request)
            .wrap_fn(track_request)
            .wrap_fn(request_id_injector)
            .wrap(session(&session_key[..], https))
            .wrap(Compress::default())
            .wrap(Logger::default())
            .service(user::reset_key)
            .service(user::user)
            .service(user::update)
            .service(user::callback)
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
            .service(Files::new("/", "./static/").index_file("index.html"))
    })
    .bind(config.bind_addr())?
    .run()
    .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Response {
    request_id: Uuid,
    success: bool,
    error_message: String,
    hint: String,
}

fn migrate(config: &PipeHubConfig) -> () {
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

fn client(config: &PipeHubConfig) -> BasicClient {
    let github_client_id = ClientId::new(config.github.client_id.clone());
    let github_client_secret = ClientSecret::new(config.github.client_secret.clone());
    let auth_url = AuthUrl::new(config.github.auth_url());
    let token_url = TokenUrl::new(config.github.token_url());

    BasicClient::new(
        github_client_id,
        Some(github_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_url(RedirectUrl::new(config.github.callback_url()))
}

fn request_id_injector<
    B: MessageBody,
    S: Service<Response = ServiceResponse<B>, Request = ServiceRequest, Error = AWError>,
>(
    req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = std::result::Result<ServiceResponse<B>, AWError>> {
    let request_id = Uuid::new_v4();
    req.extensions_mut().insert(request_id);
    srv.call(req)
}

fn track_request<
    S: Service<Response = ServiceResponse<Body>, Request = ServiceRequest, Error = AWError>,
>(
    req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = std::result::Result<ServiceResponse<Body>, AWError>> {
    let logger: Data<Arc<ApplicationLogger>> =
        req.app_data().expect("No logger found in app_data().");
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");
    let method = req.method().to_string();
    // Remove the query part from the log.
    let uri = Uri::from_str(req.uri().path()).expect("Uri not found.");
    let start = Instant::now();
    let future = srv.call(req);
    async move {
        let mut res: std::result::Result<ServiceResponse<Body>, AWError> = future.await;
        let duration = start.elapsed();
        match res {
            Ok(ref response)
                if response.status() != StatusCode::BAD_REQUEST
                    && !response.status().is_server_error() =>
            {
                logger.track_request(
                    request_id,
                    &method,
                    uri,
                    duration,
                    response.status().as_str(),
                );
            }
            Ok(ref response) => {
                let error_message = response
                    .response()
                    .extensions()
                    .get::<String>()
                    .cloned()
                    .unwrap_or_else(|| "Unexpected error occurred.".to_owned());
                let status = response.status();
                logger.track_trace(request_id, Level::Error, &error_message);
                let status_str = response.status().to_string();

                logger.track_request(request_id, &method, uri, duration, &status_str);
                res = res.map(|res| {
                    res.into_response(json(
                        HttpResponse::new(status),
                        &Response {
                            request_id,
                            success: !status.is_server_error(),
                            error_message,
                            hint: HINT.to_owned(),
                        },
                    ))
                })
            }
            Err(_) => unimplemented!("Should not reach here."),
        }
        res
    }
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

fn json<T: Serialize>(mut resp: HttpResponse, value: &T) -> HttpResponse {
    match serde_json::to_string(value) {
        Ok(body) => {
            resp.headers_mut()
                .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

            resp.set_body(Body::from(body))
        }
        Err(e) => AWError::from(Error::from(e)).into(),
    }
}

fn http_client() -> Client {
    ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(5))
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
