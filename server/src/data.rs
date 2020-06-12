use crate::error::{Error, Result};
use crate::models::{Tenant, WechatWork};
use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures_util::future::{err, ok, BoxFuture, Ready};
use sqlx::cursor::HasCursor;
use sqlx::describe::Describe;
use sqlx::executor::RefExecutor;
use sqlx::postgres::PgCursor;
use sqlx::{Cursor, Execute, Executor, PgPool, Postgres};

#[derive(Debug, Clone)]
pub struct Pool {
    inner: PgPool,
}

impl<'c> RefExecutor<'c> for &'c Pool {
    type Database = Postgres;

    fn fetch_by_ref<'q, E>(self, query: E) -> <Self::Database as HasCursor<'c, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        self.inner.fetch_by_ref(query)
    }
}

impl<'p> Executor for &'p Pool {
    type Database = Postgres;

    fn execute<'e, 'q: 'e, 'c: 'e, E: 'e>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'e, std::result::Result<u64, sqlx::Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move { (&self.inner).execute(query).await })
    }

    fn fetch<'e, 'q, E>(&'e mut self, query: E) -> <Self::Database as HasCursor<'_, 'q>>::Cursor
    where
        E: Execute<'q, Postgres>,
    {
        PgCursor::from_pool(&self.inner, query)
    }

    #[doc(hidden)]
    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, std::result::Result<Describe<Self::Database>, sqlx::Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move { (&self.inner).describe(query).await })
    }
}

impl FromRequest for Pool {
    type Error = Error;
    type Future = Ready<Result<Self>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(pool) = req.app_data::<Pool>() {
            ok(pool.clone())
        } else {
            err(Error::Unexpected("No db pool found.".to_string()))
        }
    }
}

impl Pool {
    pub async fn new(conn_str: &str) -> Result<Pool> {
        let num_cpus = num_cpus::get() as u32;

        let inner = PgPool::builder().max_size(num_cpus).build(conn_str).await?;

        Ok(Pool { inner })
    }

    pub async fn find_tenant_by_id(&self, tenant_id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as!(Tenant, "SELECT * FROM tenants WHERE id = $1", tenant_id)
            .fetch_optional(self)
            .await?;

        Ok(tenant)
    }

    pub async fn find_tenant_by_github_id(&self, github_id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as!(
            Tenant,
            "SELECT * FROM tenants WHERE github_id = $1",
            github_id
        )
        .fetch_optional(self)
        .await?;

        Ok(tenant)
    }

    pub async fn find_tenant_by_app_id(&self, app_id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as!(Tenant, "SELECT * FROM tenants WHERE app_id = $1", app_id)
            .fetch_optional(self)
            .await?;

        Ok(tenant)
    }

    pub async fn insert_tenant(&self, tenant: Tenant) -> Result<Tenant> {
        let tenant = sqlx::query_as!(
            Tenant,
            "INSERT INTO tenants (app_id, github_login, github_id) VALUES ($1, $2, $3) RETURNING *",
            tenant.app_id,
            tenant.github_login,
            tenant.github_id
        )
        .fetch_one(self)
        .await?;

        Ok(tenant)
    }

    pub async fn update_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query!(
            "UPDATE tenants SET app_id = $1, block_list = $2 WHERE id = $3",
            tenant.app_id,
            tenant.block_list,
            tenant.id
        )
        .execute(self)
        .await?;

        Ok(())
    }

    pub async fn find_wechat_by_id(&self, tenant_id: i64) -> Result<Option<WechatWork>> {
        let wechat_work = sqlx::query_as!(
            WechatWork,
            "SELECT * FROM wechat_works WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_optional(self)
        .await?;

        Ok(wechat_work)
    }

    pub async fn upsert_wechat(&self, new_wechat: WechatWork) -> Result<()> {
        sqlx::query!(
            "INSERT INTO wechat_works (tenant_id, corp_id, agent_id, secret)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (tenant_id)
                 DO UPDATE SET corp_id  = $2,
                               agent_id = $3,
                               secret   = $4
            ",
            new_wechat.tenant_id,
            new_wechat.corp_id,
            new_wechat.agent_id,
            new_wechat.secret
        )
        .execute(self)
        .await?;

        Ok(())
    }

    pub async fn find_wechat_by_app_id(&self, app_id: i64) -> Result<Option<WechatWork>> {
        let wechat_work = sqlx::query_as!(
            WechatWork,
            "SELECT wechat_works.* FROM wechat_works LEFT JOIN tenants ON wechat_works.tenant_id = tenants.id WHERE app_id = $1",
            app_id
        )
        .fetch_optional(self)
        .await?;

        Ok(wechat_work)
    }
}
