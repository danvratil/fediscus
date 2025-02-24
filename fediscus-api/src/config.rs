use fediscus_common::http_server::{HttpServerConfig, HttpServerConfigError};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("Error in HTTP server configuration: {0}")]
    HttpServerConfigurationError(HttpServerConfigError),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub http_server: HttpServerConfig,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        Self::load_from("config/application")
    }

    pub fn load_from(path: &str) -> Result<Self, config::ConfigError> {
        let cfg: Config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("FEDISCUS"))
            .build()?
            .try_deserialize()?;

        cfg.http_server.validate().map_err(|e| {
            config::ConfigError::Message(ConfigError::HttpServerConfigurationError(e).to_string())
        })?;

        Ok(cfg)
    }
}
