use diesel::Insertable;
use diesel::Queryable;
use serde::Deserialize;
use serde::Serialize;

use crate::schema::*;
use base58::ToBase58;
use std::env;

#[derive(Queryable, Serialize, AsChangeset, Clone)]
pub struct Tenant {
    #[serde(skip)]
    pub id: i64,
    #[serde(skip)]
    pub app_id: i64,
    pub github_login: String,
    pub github_id: i64,
}

#[derive(Serialize)]
pub struct UserTenant {
    #[serde(flatten)]
    tenant: Tenant,
    app_key: String,
    callback_url: String,
}

#[table_name = "tenants"]
#[derive(Insertable)]
pub struct NewTenant {
    app_id: i64,
    github_login: String,
    github_id: i64,
}

impl From<Tenant> for UserTenant {
    fn from(t: Tenant) -> Self {
        let app_key = t.app_id.to_le_bytes().to_base58();
        let domain = env::var("pipehub_domain").unwrap();
        let callback_url = format!("{}/send/{}", domain, app_key);
        UserTenant {
            tenant: t,
            app_key,
            callback_url,
        }
    }
}

impl Tenant {
    pub fn new(app_id: i64, github_login: String, github_id: i64) -> Self {
        Tenant {
            id: 0,
            app_id,
            github_login,
            github_id,
        }
    }

    pub fn inserter(self) -> NewTenant {
        NewTenant {
            app_id: self.app_id,
            github_login: self.github_login,
            github_id: self.github_id,
        }
    }
}

#[derive(Queryable, Serialize, Deserialize, Default)]
pub struct WechatWork {
    #[serde(skip)]
    pub id: i64,
    #[serde(skip)]
    pub tenant_id: i64,
    pub corp_id: String,
    pub agent_id: i64,
    pub secret: String,
}

#[table_name = "wechat_works"]
#[derive(Insertable)]
pub struct NewWechatWork {
    pub tenant_id: i64,
    pub corp_id: String,
    pub agent_id: i64,
    pub secret: String,
}

impl WechatWork {
    pub fn inserter(self) -> NewWechatWork {
        NewWechatWork {
            tenant_id: self.tenant_id,
            corp_id: self.corp_id,
            agent_id: self.agent_id,
            secret: self.secret,
        }
    }
}
