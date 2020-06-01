use crate::models::{Tenant, UserTenant};
use crate::{data, error::Result, DbPool};
use actix_http::body::Body;
use actix_http::http::header;
use actix_session::Session;
use actix_web::client::Client;
use actix_web::error::Error as AWError;
use actix_web::{get, web, HttpResponse};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::prelude::SecretNewType;
use oauth2::{AuthorizationCode, CsrfToken, TokenResponse};
use serde::Deserialize;
use std::sync::Arc;

pub const TENANT_ID_KEY: &'static str = "tenant_id";
pub const STATE_KEY: &'static str = "state";

#[get("/user")]
pub async fn user(
    session: Session,
    client: web::Data<Arc<BasicClient>>,
    pool: web::Data<DbPool>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(tenant) = data::find_tenant_by_id(pool, tenant_id).await? {
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
    pool: web::Data<DbPool>,
    web::Query(callback): web::Query<Callback>,
) -> std::result::Result<HttpResponse, AWError> {
    match session.get::<CsrfToken>(STATE_KEY)? {
        Some(state) if state == callback.state => {
            let code = AuthorizationCode::new(callback.code);
            let token = web::block(move || -> Result<BasicTokenResponse> {
                let code = client.exchange_code(code)?;
                Ok(code)
            })
            .await?;

            let access_token = token.access_token().secret();
            let http_client = Client::default();
            let mut response = http_client
                .get("https://api.github.com/user")
                .header(header::USER_AGENT, "PipeHub")
                .header(header::AUTHORIZATION, format!("token {}", access_token))
                .send()
                .await?;
            let github_user = response.json::<GithubUser>().await?;
            match data::find_tenant_by_github_id(pool.clone(), github_user.id).await? {
                Some(tenant) => session.set(TENANT_ID_KEY, tenant.id)?,
                None => {
                    let app_id: i64 = rand::random();
                    let tenant = Tenant::new(app_id, github_user.login, github_user.id);
                    let tenant = data::insert_tenant(pool.clone(), tenant).await?;
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

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
    id: i64,
}
