use crate::error::{Error, Result};
use crate::logger::ApplicationLogger;
use crate::models::{Tenant, UserTenant};
use crate::{data, DbPool};
use actix_http::body::Body;
use actix_http::http::header;
use actix_session::Session;
use actix_web::error::Error as AWError;
use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::prelude::SecretNewType;
use oauth2::{AuthorizationCode, CsrfToken, TokenResponse};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub const TENANT_ID_KEY: &str = "tenant_id";
pub const STATE_KEY: &str = "state";

#[get("/user")]
pub async fn user(
    session: Session,
    client: web::Data<Arc<BasicClient>>,
    pool: web::Data<DbPool>,
    logger: web::Data<Arc<ApplicationLogger>>,
    req: HttpRequest,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) =
            data::find_tenant_by_id(request_id, Arc::clone(&logger), pool, tenant_id).await?
        {
            return Ok(HttpResponse::Ok().json(UserTenant::from(tenant)));
        };
    }

    let (url, token) = client.authorize_url(CsrfToken::new_random);
    session.set(STATE_KEY, token)?;
    Ok(HttpResponse::Unauthorized()
        .header("Location", url.to_string())
        .body(Body::Empty))
}

#[derive(Deserialize)]
pub struct Callback {
    code: String,
    state: CsrfToken,
}

#[get("/callback")]
pub async fn callback(
    session: Session,
    client: web::Data<Arc<BasicClient>>,
    http_client: web::Data<Client>,
    pool: web::Data<DbPool>,
    web::Query(callback): web::Query<Callback>,
    logger: web::Data<Arc<ApplicationLogger>>,
    req: HttpRequest,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");
    match session.get::<CsrfToken>(STATE_KEY)? {
        Some(state) if state == callback.state => {
            let code = AuthorizationCode::new(callback.code);
            let logger_move = logger.clone();
            let start = Instant::now();
            let success;
            let result_code;
            let token = web::block(move || -> Result<BasicTokenResponse> {
                let code = client.exchange_code(code)?;
                Ok(code)
            })
            .await;
            match token {
                Ok(_) => {
                    success = true;
                    result_code = "".to_owned();
                }
                Err(ref e) => {
                    success = false;
                    result_code = format!("{:?}", e);
                }
            };

            logger_move.track_dependency(
                request_id,
                "OAuth exchange_code",
                "OAuth",
                start.elapsed(),
                "GitHub",
                &result_code,
                "",
                success,
            );
            let token = token.map_err(Error::from)?;

            let access_token = token.access_token().secret();
            let start = Instant::now();
            let response = http_client
                .get("https://api.github.com/user")
                .header(header::USER_AGENT, "PipeHub")
                .header(header::AUTHORIZATION, format!("token {}", access_token))
                .send()
                .await
                .map_err(Error::from)?;
            logger.track_dependency(
                request_id,
                "GET https://api.github.com/user",
                "HTTPS",
                start.elapsed(),
                "api.github.com",
                response.status().as_str(),
                access_token,
                response.status().is_success(),
            );
            let github_user = response.json::<GithubUser>().await.map_err(Error::from)?;
            match data::find_tenant_by_github_id(
                request_id,
                Arc::clone(&logger),
                pool.clone(),
                github_user.id,
            )
            .await?
            {
                Some(tenant) => session.set(TENANT_ID_KEY, tenant.id)?,
                None => {
                    let app_id: i64 = rand::random();
                    let tenant = Tenant::new(app_id, github_user.login, github_user.id);
                    let tenant =
                        data::insert_tenant(request_id, Arc::clone(&logger), pool.clone(), tenant)
                            .await?;
                    session.set(TENANT_ID_KEY, tenant.id)?
                }
            }
            Ok(HttpResponse::Found()
                .header("Location", "/#/user")
                .body(Body::Empty))
        }
        _ => Ok(HttpResponse::Found()
            .header("Location", "/")
            .body(Body::Empty)),
    }
}

#[post("/user/reset_key")]
pub async fn reset_key(
    session: Session,
    pool: web::Data<DbPool>,
    logger: web::Data<Arc<ApplicationLogger>>,
    req: HttpRequest,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");

    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) =
            data::find_tenant_by_id(request_id, Arc::clone(&logger), pool.clone(), tenant_id)
                .await?
        {
            let new_tenant = Tenant {
                id: tenant.id,
                app_id: rand::random(),
                github_login: tenant.github_login,
                github_id: tenant.github_id,
                block_list: tenant.block_list,
            };
            data::update_tenant(
                request_id,
                Arc::clone(&logger),
                pool.clone(),
                new_tenant.clone(),
            )
            .await?;

            return Ok(HttpResponse::Ok().json(UserTenant::from(new_tenant)));
        };
    }

    Ok(HttpResponse::Unauthorized().body(Body::Empty))
}

#[put("/user")]
pub async fn update(
    session: Session,
    pool: web::Data<DbPool>,
    logger: web::Data<Arc<ApplicationLogger>>,
    req: HttpRequest,
    web::Json(new_tenant): web::Json<Tenant>,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");

    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) =
            data::find_tenant_by_id(request_id, Arc::clone(&logger), pool.clone(), tenant_id)
                .await?
        {
            let new_tenant = Tenant {
                id: tenant.id,
                app_id: tenant.app_id,
                github_login: tenant.github_login,
                github_id: tenant.github_id,
                block_list: new_tenant.block_list,
            };
            data::update_tenant(
                request_id,
                Arc::clone(&logger),
                pool.clone(),
                new_tenant.clone(),
            )
            .await?;

            return Ok(HttpResponse::Ok().json(UserTenant::from(new_tenant)));
        };
    }

    Ok(HttpResponse::Unauthorized().body(Body::Empty))
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
    id: i64,
}
