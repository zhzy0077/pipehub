#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::config::PipeHubConfig;
use crate::error::Result;
use actix_files::Files;
use actix_session::CookieSession;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use r2d2::PooledConnection;
use std::io;
use std::sync::Arc;

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

embed_migrations!("./migrations");

#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let config = PipeHubConfig::new()?;

    migrate(&config);

    logger::configure_logger(&config.log).await?;

    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool: DbPool = Pool::new(manager)?;

    let session_key: [u8; 32] = rand::random();
    let github_client = Arc::new(client(&config));
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(github_client.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(session(&session_key[..]))
            .service(user::user)
            .service(user::callback)
            .service(wechat::wechat)
            .service(wechat::update)
            .service(
                web::resource("/send/{key}")
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

fn migrate(config: &PipeHubConfig) -> () {
    let connection =
        PgConnection::establish(&config.database_url).expect("Unable to connect to DB.");

    embedded_migrations::run_with_output(&connection, &mut io::stdout())
        .expect("Unable to migrate.");
}

fn session(key: &[u8]) -> CookieSession {
    CookieSession::private(key)
        .name("session")
        .secure(false)
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
