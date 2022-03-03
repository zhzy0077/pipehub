use crate::error::Result;
use config::{Config, Environment};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct PipeHubConfig {
    pub host: String,
    pub port: u16,
    pub domain: String,
    pub domain_web: String,
    // If we need to make cookie secure.
    pub https: bool,
    pub database_url: String,
    pub github: GitHubConfig,
    pub microsoft: MicrosoftConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MicrosoftConfig {
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
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
        let port = env::var("PORT").unwrap_or(self.port.to_string());
        format!("{}:{}", self.host, port)
    }
}
