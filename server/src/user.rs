use crate::data::Pool;
use crate::github::GitHubClient;
use crate::models::{Tenant, UserTenant};
use actix_http::body::Body;
use actix_session::Session;
use actix_web::error::Error as AWError;
use actix_web::{get, post, put, web, HttpResponse};
use base58::ToBase58;
use log::info;

use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::Deserialize;

pub const TENANT_ID_KEY: &str = "tenant_id";
pub const STATE_KEY: &str = "state";

#[get("/user")]
pub async fn user(
    session: Session,
    client: web::Data<GitHubClient>,
    pool: web::Data<Pool>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) = pool.find_tenant_by_id(tenant_id).await? {
            return Ok(HttpResponse::Ok().json(UserTenant::from(tenant)));
        };
    }

    let state = new_csrf_token();
    let url = client.authorize_url(&state);
    info!("Setting {}", state.clone());
    session.set(STATE_KEY, state)?;
    Ok(HttpResponse::Unauthorized()
        .header("Location", url.to_string())
        .body("hello world"))
}

fn new_csrf_token() -> String {
    let random_bytes: [u8; 16] = thread_rng().gen::<[u8; 16]>();
    random_bytes.to_base58()
}

#[derive(Deserialize)]
pub struct Callback {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct LoginCallback {
    access_token: String,
}

// It's for testing purpose.
#[post("/login")]
pub async fn login(
    session: Session,
    http_client: web::Data<Client>,
    github_client: web::Data<GitHubClient>,
    pool: web::Data<Pool>,
    web::Query(login): web::Query<LoginCallback>,
) -> std::result::Result<HttpResponse, AWError> {
    let access_token = login.access_token;
    let github_user = github_client.get_user(&http_client, &access_token).await?;
    match pool.find_tenant_by_github_id(github_user.id).await? {
        Some(tenant) => session.set(TENANT_ID_KEY, tenant.id)?,
        None => {
            let app_id: i64 = thread_rng().gen();
            let tenant = Tenant::new(app_id, github_user.login, github_user.id);
            let tenant = pool.insert_tenant(tenant).await?;
            session.set(TENANT_ID_KEY, tenant.id)?
        }
    }
    Ok(HttpResponse::Found()
        .header("Location", "/#/user")
        .body("hello world"))
}

#[get("/callback")]
pub async fn callback(
    session: Session,
    github_client: web::Data<GitHubClient>,
    http_client: web::Data<Client>,
    pool: web::Data<Pool>,
    web::Query(callback): web::Query<Callback>,
) -> std::result::Result<HttpResponse, AWError> {
    info!(
        "{:?} - {:?}",
        session.get::<String>(STATE_KEY),
        callback.state
    );
    match session.get::<String>(STATE_KEY)? {
        Some(state) if state == callback.state => {
            let access_token = github_client
                .exchange_code(&http_client, &callback.code)
                .await?;
            info!("{:?}", access_token);
            let github_user = github_client.get_user(&http_client, &access_token).await?;

            info!("{:?}", github_user);

            match pool.find_tenant_by_github_id(github_user.id).await? {
                Some(tenant) => session.set(TENANT_ID_KEY, tenant.id)?,
                None => {
                    let app_id: i64 = thread_rng().gen();
                    let tenant = Tenant::new(app_id, github_user.login, github_user.id);
                    let tenant = pool.insert_tenant(tenant).await?;
                    session.set(TENANT_ID_KEY, tenant.id)?
                }
            }
            Ok(HttpResponse::Found()
                .header("Location", "http://localhost:3000/#/user")
                .body(Body::Empty))
        }
        _ => Ok(HttpResponse::Found()
            .header("Location", "http://localhost:3000/")
            .body(Body::Empty)),
    }
}

#[post("/user/reset_key")]
pub async fn reset_key(
    session: Session,
    pool: web::Data<Pool>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) = pool.find_tenant_by_id(tenant_id).await? {
            let new_tenant = Tenant {
                id: tenant.id,
                app_id: thread_rng().gen(),
                github_login: tenant.github_login,
                github_id: tenant.github_id,
                block_list: tenant.block_list,
            };
            pool.update_tenant(new_tenant.clone()).await?;

            return Ok(HttpResponse::Ok().json(UserTenant::from(new_tenant)));
        };
    }

    Ok(HttpResponse::Unauthorized().body(Body::Empty))
}

#[put("/user")]
pub async fn update(
    session: Session,
    pool: web::Data<Pool>,
    web::Json(new_tenant): web::Json<Tenant>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) = pool.find_tenant_by_id(tenant_id).await? {
            let new_tenant = Tenant {
                id: tenant.id,
                app_id: tenant.app_id,
                github_login: tenant.github_login,
                github_id: tenant.github_id,
                block_list: new_tenant.block_list,
            };
            pool.update_tenant(new_tenant.clone()).await?;

            return Ok(HttpResponse::Ok().json(UserTenant::from(new_tenant)));
        };
    }

    Ok(HttpResponse::Unauthorized().body(Body::Empty))
}
