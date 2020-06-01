use crate::error::Result;
use config::{Config, Environment};
use log::Level;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct PipeHubConfig {
    pub host: String,
    pub port: u16,
    pub domain: String,
    pub database_url: String,
    pub github: GitHubConfig,
    pub log: LogConfig,
}

#[derive(Debug, Deserialize)]
pub struct GitHubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub callback_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub instrumentation_key: String,
    pub logger: Logger,
    pub level: Level,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Logger {
    ApplicationInsight,
    TermLogger,
}

impl PipeHubConfig {
    pub fn new() -> Result<Self> {
        let environment = Environment::new().prefix("pipehub").separator("__");
        let mut config = Config::new();

        config.merge(environment)?;
        let config = config.try_into()?;

        Ok(config)
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl GitHubConfig {
    pub fn auth_url(&self) -> Url {
        Url::parse(&self.auth_url).unwrap()
    }
    pub fn token_url(&self) -> Url {
        Url::parse(&self.token_url).unwrap()
    }
    pub fn callback_url(&self) -> Url {
        Url::parse(&self.callback_url).unwrap()
    }
}
