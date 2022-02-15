use crate::error::Result;
use crate::models::{Tenant, WechatWork};
use log::LevelFilter;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};

#[derive(Debug, Clone)]
pub struct Pool {
    inner: PgPool,
}

impl Pool {
    pub async fn new(conn_str: &str) -> Result<Pool> {
        let num_cpus = num_cpus::get() as u32;

        let mut connect_options = conn_str.parse::<PgConnectOptions>()?;

        connect_options.log_statements(LevelFilter::Debug);
        connect_options.log_slow_statements(LevelFilter::Info, Duration::from_secs(1));

        let inner = PgPoolOptions::new()
            .max_connections(num_cpus)
            .connect_with(connect_options)
            .await?;

        Ok(Pool { inner })
    }

    pub async fn migrate(&self) -> Result<()> {
        return Ok(sqlx::migrate!("./migrations").run(&self.inner).await?);
    }

    pub async fn find_tenant_by_id(&self, tenant_id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE id = $1")
            .bind(tenant_id)
            .fetch_optional(&self.inner)
            .await?;

        Ok(tenant)
    }

    pub async fn find_tenant_by_github_id(&self, github_id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE github_id = $1")
            .bind(github_id)
            .fetch_optional(&self.inner)
            .await?;

        Ok(tenant)
    }

    pub async fn find_tenant_by_app_id(&self, app_id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE app_id = $1")
            .bind(app_id)
            .fetch_optional(&self.inner)
            .await?;

        Ok(tenant)
    }

    pub async fn insert_tenant(&self, tenant: Tenant) -> Result<Tenant> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "INSERT INTO tenants (app_id, github_login, github_id) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(tenant.app_id)
        .bind(tenant.github_login)
        .bind(tenant.github_id)
        .fetch_one(&self.inner)
        .await?;

        Ok(tenant)
    }

    pub async fn update_tenant(&self, tenant: Tenant) -> Result<()> {
        sqlx::query("UPDATE tenants SET app_id = $1, block_list = $2 WHERE id = $3")
            .bind(tenant.app_id)
            .bind(tenant.block_list)
            .bind(tenant.id)
            .execute(&self.inner)
            .await?;

        Ok(())
    }

    pub async fn find_wechat_by_id(&self, tenant_id: i64) -> Result<Option<WechatWork>> {
        let wechat_work =
            sqlx::query_as::<_, WechatWork>("SELECT * FROM wechat_works WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_optional(&self.inner)
                .await?;

        Ok(wechat_work)
    }

    pub async fn upsert_wechat(&self, new_wechat: WechatWork) -> Result<()> {
        sqlx::query(
            "INSERT INTO wechat_works (tenant_id, corp_id, agent_id, secret, bot_token, chat_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (tenant_id)
                 DO UPDATE SET corp_id   = $2,
                               agent_id  = $3,
                               secret    = $4,
                               bot_token = $5,
                               chat_id   = $6
            ",
        )
        .bind(new_wechat.tenant_id)
        .bind(new_wechat.corp_id)
        .bind(new_wechat.agent_id)
        .bind(new_wechat.secret)
        .bind(new_wechat.bot_token)
        .bind(new_wechat.chat_id)
        .execute(&self.inner)
        .await?;

        Ok(())
    }

    pub async fn find_wechat_by_app_id(&self, app_id: i64) -> Result<Option<WechatWork>> {
        let wechat_work = sqlx::query_as::<_, WechatWork>("SELECT wechat_works.* FROM wechat_works LEFT JOIN tenants ON wechat_works.tenant_id = tenants.id WHERE app_id = $1")
            .bind(app_id)
        .fetch_optional(&self.inner)
        .await?;

        Ok(wechat_work)
    }
}
