use crate::error::Result;
use crate::models::{Tenant, WechatWork};
use crate::DbPool;
use actix_web::web;
use diesel::debug_query;
use diesel::insert_into;
use diesel::pg::Pg;
use diesel::prelude::*;
use log::debug;

pub(crate) async fn find_tenant_by_id(
    pool: web::Data<DbPool>,
    tenant_id: i64,
) -> Result<Option<Tenant>> {
    use crate::schema::tenants::dsl::*;
    let conn = pool.get()?;
    let tenant = web::block(move || -> Result<Option<Tenant>> {
        let query = tenants.filter(id.eq(tenant_id));
        debug!("SQL: {}.", debug_query::<Pg, _>(&query));
        let tenant = query.first::<Tenant>(&conn).optional()?;
        Ok(tenant)
    })
    .await?;

    Ok(tenant)
}

pub(crate) async fn find_tenant_by_github_id(
    pool: web::Data<DbPool>,
    tenant_github_id: i64,
) -> Result<Option<Tenant>> {
    use crate::schema::tenants::dsl::*;
    let conn = pool.get()?;
    let tenant = web::block(move || -> Result<Option<Tenant>> {
        let query = tenants.filter(github_id.eq(tenant_github_id));
        debug!("SQL: {}.", debug_query::<Pg, _>(&query));
        let tenant = query.first::<Tenant>(&conn).optional()?;
        Ok(tenant)
    })
    .await?;

    Ok(tenant)
}

pub(crate) async fn insert_tenant(pool: web::Data<DbPool>, tenant: Tenant) -> Result<Tenant> {
    use crate::schema::tenants::dsl::*;
    let conn = pool.get()?;
    let tenant = web::block(move || -> Result<Tenant> {
        let inserter = tenant.inserter();
        let query = insert_into(tenants).values(&inserter);
        debug!("SQL: {}.", debug_query::<Pg, _>(&query));
        let tenant = query.get_result(&conn)?;
        Ok(tenant)
    })
    .await?;

    Ok(tenant)
}

pub(crate) async fn find_wechat_by_id(
    pool: web::Data<DbPool>,
    wechat_tenant_id: i64,
) -> Result<Option<WechatWork>> {
    use crate::schema::wechat_works::dsl::*;
    let conn = pool.get()?;
    let wechat = web::block(move || -> Result<Option<WechatWork>> {
        let query = wechat_works.filter(tenant_id.eq(wechat_tenant_id));
        debug!("SQL: {}.", debug_query::<Pg, _>(&query));
        let wechat = query.first::<WechatWork>(&conn).optional()?;
        Ok(wechat)
    })
    .await?;

    Ok(wechat)
}

pub(crate) async fn upsert_wechat(pool: web::Data<DbPool>, new_wechat: WechatWork) -> Result<()> {
    use crate::schema::wechat_works::dsl::*;
    let conn = pool.get()?;
    web::block(move || -> Result<()> {
        conn.transaction(|| -> Result<()> {
            let query = wechat_works.filter(tenant_id.eq(new_wechat.tenant_id));
            debug!("SQL: {}.", debug_query::<Pg, _>(&query));
            let wechat: Option<WechatWork> = query.first::<WechatWork>(&conn).optional()?;
            if let Some(wechat) = wechat {
                let query = diesel::update(wechat_works.filter(tenant_id.eq(wechat.tenant_id)))
                    .set((
                        corp_id.eq(new_wechat.corp_id),
                        agent_id.eq(new_wechat.agent_id),
                        secret.eq(new_wechat.secret),
                    ));
                debug!("SQL: {}.", debug_query::<Pg, _>(&query));
                query.execute(&conn)?;
            } else {
                let query = diesel::insert_into(wechat_works).values(new_wechat.inserter());
                debug!("SQL: {}.", debug_query::<Pg, _>(&query));
                query.execute(&conn)?;
            }
            Ok(())
        })?;

        Ok(())
    })
    .await?;

    Ok(())
}

pub(crate) async fn find_wechat_by_app_id(
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
        debug!("SQL: {}.", debug_query::<Pg, _>(&query));
        let data: Option<WechatWork> = query.first::<WechatWork>(&conn).optional()?;

        Ok(data)
    })
    .await?;

    Ok(wechat)
}
