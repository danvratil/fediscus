// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use serde::Deserialize;

const fn default_false() -> bool {
    false
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The HTTP server configuration
pub struct HttpServer {
    /// The address and port to bind to
    pub listen: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The HTTP client configuration
pub struct HttpClient {
    /// The user agent to use for requests
    pub user_agent: Option<String>,

    #[serde(default = "default_false")]
    pub allow_http: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The database configuration
pub struct Database {
    /// The URL to connect to the database
    pub url: String,
    /// The maximum number of connections to keep in the pool
    pub pool_size: Option<u32>,
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
    pub http_server: HttpServer,
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
        config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("FEDISCUS"))
            .build()?
            .try_deserialize()
    }
}
