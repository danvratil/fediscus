// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

//! Configuration management for the Fediscus ActivityPub server
//!
//! This module handles all configuration options for the server, including
//! HTTP server settings, client settings, and database configuration.
//! Configuration can be loaded from files and environment variables.

use fediscus_common::http_server::{HttpServerConfig, HttpServerConfigError};
use serde::Deserialize;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("Error in HTTP server configuration: {0}")]
    HttpServerConfigurationError(HttpServerConfigError),
    #[error("Invalid database URL: {0}")]
    InvalidDatabaseUrl(String),
    #[error("Pool size must be greater than 0")]
    InvalidPoolSize,
}

const fn default_false() -> bool {
    false
}

const fn default_pool_size() -> u32 {
    10
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The HTTP client configuration
pub struct HttpClient {
    /// The user agent to use for requests. If not specified,
    /// defaults to "fediscus-activitypub/VERSION"
    pub user_agent: Option<String>,

    /// Whether to allow HTTP connections (default: false)
    /// Should only be enabled for testing
    #[serde(default = "default_false")]
    pub allow_http: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The database configuration
pub struct Database {
    /// The URL to connect to the database
    /// Supports PostgreSQL URLs in format:
    /// "postgres://user:pass@host:port/dbname"
    pub url: String,

    /// The maximum number of connections to keep in the pool
    /// Defaults to 10 if not specified
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

impl Database {
    /// Validates the database configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate database URL
        Url::parse(&self.url).map_err(|e| ConfigError::InvalidDatabaseUrl(e.to_string()))?;

        // Validate pool size
        if self.pool_size == 0 {
            return Err(ConfigError::InvalidPoolSize);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The main service user exposed as federated user
/// Only the main fields are exposed, the rest is either omitted (because irrelevant) or
/// will be loaded from the database
pub struct FediverseUser {
    /// The username of the user
    pub username: String,
    /// Name of the local instance
    pub host: String,
    /// The display name of the user
    pub display_name: String,
    /// The RSA private key of the user in PEM format, the public key will be derived automatically
    pub private_key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The main configuration struct
pub struct Config {
    /// The HTTP server configuration
    pub http_server: HttpServerConfig,
    /// The HTTP client configuration
    pub http_client: HttpClient,
    /// The database configuration
    pub database: Database,
    /// The main service user exposed as federated user
    pub fediverse_user: FediverseUser,
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

        cfg.http_server
            .validate()
            .map_err(|e| config::ConfigError::Message(e.to_string()))?;
        cfg.database
            .validate()
            .map_err(|e| config::ConfigError::Message(e.to_string()))?;

        Ok(cfg)
    }
}
