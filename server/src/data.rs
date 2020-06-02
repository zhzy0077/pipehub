use crate::error::Result;
use crate::logger::ApplicationLogger;
use crate::models::{Tenant, WechatWork};
use crate::DbPool;
use actix_web::web;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::{debug_query, insert_into};
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub(crate) async fn find_tenant_by_id(
    request_id: Uuid,
    logger: Arc<ApplicationLogger>,
    pool: web::Data<DbPool>,
    tenant_id: i64,
) -> Result<Option<Tenant>> {
    use crate::schema::tenants::dsl::*;
    let conn = pool.get()?;
    let tenant = web::block(move || -> Result<Option<Tenant>> {
        let query = tenants.filter(id.eq(tenant_id));
        let start = Instant::now();
        let tenant = query.first::<Tenant>(&conn).optional()?;
        log_query(
            request_id,
            &logger,
            "TENANT",
            debug_query::<Pg, _>(&query),
            start.elapsed(),
            true,
        );
        Ok(tenant)
    })
    .await?;

    Ok(tenant)
}

pub(crate) async fn find_tenant_by_github_id(
    request_id: Uuid,
    logger: Arc<ApplicationLogger>,
    pool: web::Data<DbPool>,
    tenant_github_id: i64,
) -> Result<Option<Tenant>> {
    use crate::schema::tenants::dsl::*;
    let conn = pool.get()?;
    let tenant = web::block(move || -> Result<Option<Tenant>> {
        let query = tenants.filter(github_id.eq(tenant_github_id));
        let start = Instant::now();
        let tenant = query.first::<Tenant>(&conn).optional()?;
        log_query(
            request_id,
            &logger,
            "TENANT",
            debug_query::<Pg, _>(&query),
            start.elapsed(),
            true,
        );
        Ok(tenant)
    })
    .await?;

    Ok(tenant)
}

pub(crate) async fn insert_tenant(
    request_id: Uuid,
    logger: Arc<ApplicationLogger>,
    pool: web::Data<DbPool>,
    tenant: Tenant,
) -> Result<Tenant> {
    use crate::schema::tenants::dsl::*;
    let conn = pool.get()?;
    let tenant = web::block(move || -> Result<Tenant> {
        let inserter = tenant.inserter();
        let query = insert_into(tenants).values(&inserter);
        let start = Instant::now();
        let tenant = query.get_result(&conn)?;
        log_query(
            request_id,
            &logger,
            "TENANT",
            debug_query::<Pg, _>(&query),
            start.elapsed(),
            true,
        );
        Ok(tenant)
    })
    .await?;

    Ok(tenant)
}

pub(crate) async fn find_wechat_by_id(
    request_id: Uuid,
    logger: Arc<ApplicationLogger>,
    pool: web::Data<DbPool>,
    wechat_tenant_id: i64,
) -> Result<Option<WechatWork>> {
    use crate::schema::wechat_works::dsl::*;
    let conn = pool.get()?;
    let wechat = web::block(move || -> Result<Option<WechatWork>> {
        let query = wechat_works.filter(tenant_id.eq(wechat_tenant_id));
        let start = Instant::now();
        let wechat = query.first::<WechatWork>(&conn).optional()?;
        log_query(
            request_id,
            &logger,
            "WECHAT",
            debug_query::<Pg, _>(&query),
            start.elapsed(),
            true,
        );
        Ok(wechat)
    })
    .await?;

    Ok(wechat)
}

pub(crate) async fn upsert_wechat(
    request_id: Uuid,
    logger: Arc<ApplicationLogger>,
    pool: web::Data<DbPool>,
    new_wechat: WechatWork,
) -> Result<()> {
    use crate::schema::wechat_works::dsl::*;
    let conn = pool.get()?;
    web::block(move || -> Result<()> {
        conn.transaction(|| -> Result<()> {
            let query = wechat_works.filter(tenant_id.eq(new_wechat.tenant_id));
            let start = Instant::now();
            let wechat: Option<WechatWork> = query.first::<WechatWork>(&conn).optional()?;
            log_query(
                request_id,
                &logger,
                "WECHAT",
                debug_query::<Pg, _>(&query),
                start.elapsed(),
                true,
            );
            if let Some(wechat) = wechat {
                let query = diesel::update(wechat_works.filter(tenant_id.eq(wechat.tenant_id)))
                    .set((
                        corp_id.eq(new_wechat.corp_id),
                        agent_id.eq(new_wechat.agent_id),
                        secret.eq(new_wechat.secret),
                    ));
                let start = Instant::now();
                let sql = debug_query::<Pg, _>(&query).to_string();
                query.execute(&conn)?;
                log_query(request_id, &logger, "WECHAT", sql, start.elapsed(), true);
            } else {
                let query = diesel::insert_into(wechat_works).values(new_wechat.inserter());
                let sql = debug_query::<Pg, _>(&query).to_string();
                let start = Instant::now();
                query.execute(&conn)?;
                log_query(request_id, &logger, "WECHAT", sql, start.elapsed(), true);
            }
            Ok(())
        })?;

        Ok(())
    })
    .await?;

    Ok(())
}

pub(crate) async fn find_wechat_by_app_id(
    request_id: Uuid,
    logger: Arc<ApplicationLogger>,
    pool: web::Data<DbPool>,
    wechat_app_id: i64,
) -> Result<Option<WechatWork>> {
    use crate::schema::tenants;
    use crate::schema::tenants::dsl::*;
    use crate::schema::wechat_works;

    let conn = pool.get()?;
    let wechat = web::block(move || -> Result<Option<WechatWork>> {
        let query = tenants::table
            .inner_join(wechat_works::table.on(tenants::id.eq(wechat_works::tenant_id)))
            .select(wechat_works::all_columns)
            .filter(app_id.eq(wechat_app_id));
        let start = Instant::now();
        let data: Option<WechatWork> = query.first::<WechatWork>(&conn).optional()?;
        log_query(
            request_id,
            &logger,
            "TENANT",
            debug_query::<Pg, _>(&query),
            start.elapsed(),
            true,
        );
        Ok(data)
    })
    .await?;

    Ok(wechat)
}

fn log_query<T>(
    request_id: Uuid,
    logger: &ApplicationLogger,
    table_name: &str,
    query: T,
    duration: Duration,
    success: bool,
) where
    T: Display,
{
    logger.track_dependency(
        request_id,
        &format!("EXECUTE {}", table_name),
        "SQL",
        duration,
        "PostgreSQL",
        "",
        &format!("{}", query),
        success,
    );
}
