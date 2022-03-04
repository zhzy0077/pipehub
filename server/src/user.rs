use crate::data::Pool;
use crate::github::GitHubClient;
use crate::models::{Tenant, UserTenant};

use crate::{MicrosoftClient, RequestId};
use actix_session::Session;
use actix_web::error::Error as AWError;
use actix_web::{get, post, put, web, HttpResponse};
use base58::ToBase58;
use log::info;
use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::Deserialize;
use std::env;

pub const TENANT_ID_KEY: &str = "tenant_id";
pub const STATE_KEY: &str = "state";

#[get("/user")]
pub async fn user(
    session: Session,
    client: web::Data<GitHubClient>,
    pool: web::Data<Pool>,
    request_id: RequestId,
) -> std::result::Result<HttpResponse, AWError> {
    info!("{} [Get User]", request_id);
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) = pool.find_tenant_by_id(tenant_id).await? {
            info!("{} [Get User] {}", request_id, tenant.id);
            return Ok(HttpResponse::Ok().json(UserTenant::from(tenant)));
        };
    }

    let state = new_csrf_token();
    let url = client.authorize_url(&state);
    session.insert(STATE_KEY, state)?;
    Ok(HttpResponse::Unauthorized()
        .append_header(("Location", url.to_string()))
        .body(()))
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
        Some(tenant) => session.insert(TENANT_ID_KEY, tenant.id)?,
        None => {
            let app_id: i64 = thread_rng().gen();
            let tenant = Tenant::new(app_id, github_user.login, github_user.id);
            let tenant = pool.insert_tenant(tenant).await?;
            session.insert(TENANT_ID_KEY, tenant.id)?
        }
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", "/#/user"))
        .body(()))
}

#[get("/msft_auth_url")]
pub async fn msft_auth_url(
    session: Session,
    client: web::Data<MicrosoftClient>,
) -> std::result::Result<HttpResponse, AWError> {
    let state = new_csrf_token();
    let url = client.authorize_url(&state);
    session.insert(STATE_KEY, state)?;
    Ok(HttpResponse::Ok()
        .append_header(("Location", url.to_string()))
        .body(()))
}

#[get("/msft_callback")]
pub async fn msft_callback(
    session: Session,
    microsoft_client: web::Data<MicrosoftClient>,
    http_client: web::Data<Client>,
    web::Query(msft_callback): web::Query<Callback>,
    pool: web::Data<Pool>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(state) = session.get::<String>(STATE_KEY)? {
        if state == msft_callback.state {
            if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
                if let Some(mut tenant) = pool.find_tenant_by_id(tenant_id).await? {
                    let refresh_token = microsoft_client
                        .exchange_refresh_code(&http_client, &msft_callback.code)
                        .await?;

                    tenant.msft_refresh_token = refresh_token;
                    pool.update_tenant(tenant).await?;
                }
            }
        }
    }
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!("{}/#/user", env::var("pipehub_domain_web").unwrap()),
        ))
        .body(()))
}

#[get("/callback")]
pub async fn callback(
    session: Session,
    github_client: web::Data<GitHubClient>,
    http_client: web::Data<Client>,
    pool: web::Data<Pool>,
    web::Query(callback): web::Query<Callback>,
    request_id: RequestId,
) -> std::result::Result<HttpResponse, AWError> {
    match session.get::<String>(STATE_KEY)? {
        Some(state) if state == callback.state => {
            let access_token = github_client
                .exchange_code(&http_client, &callback.code)
                .await?;
            let github_user = github_client.get_user(&http_client, &access_token).await?;

            match pool.find_tenant_by_github_id(github_user.id).await? {
                Some(tenant) => {
                    info!("{} [Login] {}", request_id, github_user.login);
                    session.insert(TENANT_ID_KEY, tenant.id)?
                }
                None => {
                    let app_id: i64 = thread_rng().gen();
                    info!("{} [Register] {}", request_id, github_user.login);
                    let tenant = Tenant::new(app_id, github_user.login, github_user.id);
                    let tenant = pool.insert_tenant(tenant).await?;
                    session.insert(TENANT_ID_KEY, tenant.id)?
                }
            }
            Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("{}/#/user", env::var("pipehub_domain_web").unwrap()),
                ))
                .body(()))
        }
        _ => Ok(HttpResponse::Found()
            .append_header((
                "Location",
                format!("{}/", env::var("pipehub_domain_web").unwrap()),
            ))
            .body(())),
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
                captcha: tenant.captcha,
                msft_refresh_token: tenant.msft_refresh_token,
                msft_task_list_id: tenant.msft_task_list_id,
            };
            pool.update_tenant(new_tenant.clone()).await?;

            return Ok(HttpResponse::Ok().json(UserTenant::from(new_tenant)));
        };
    }

    Ok(HttpResponse::Unauthorized().body(()))
}

#[put("/user")]
pub async fn update(
    session: Session,
    pool: web::Data<Pool>,
    web::Json(new_tenant): web::Json<Tenant>,
    request_id: RequestId,
) -> std::result::Result<HttpResponse, AWError> {
    info!("{} [Update User]", request_id);
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) = pool.find_tenant_by_id(tenant_id).await? {
            info!("{} [Update User] {}", request_id, tenant.id);
            let new_tenant = Tenant {
                id: tenant.id,
                app_id: tenant.app_id,
                github_login: tenant.github_login,
                github_id: tenant.github_id,
                block_list: new_tenant.block_list,
                captcha: new_tenant.captcha,
                msft_refresh_token: tenant.msft_refresh_token,
                msft_task_list_id: new_tenant.msft_task_list_id,
            };
            pool.update_tenant(new_tenant.clone()).await?;

            return Ok(HttpResponse::Ok().json(UserTenant::from(new_tenant)));
        };
    }

    Ok(HttpResponse::Unauthorized().body(()))
}
