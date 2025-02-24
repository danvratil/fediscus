use serde::Deserialize;
use thiserror::Error;

/// The HTTP server configuration validation errors
#[derive(Debug, Error)]
pub enum HttpServerConfigError {
    #[error("Invalid listen address: {0}")]
    InvalidListenAddress(String),
}

/// The HTTP server configuration
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HttpServerConfig {
    /// The address and port to bind to (format: "ip:port" or ":port")
    pub listen: String,
}

impl HttpServerConfig {
    /// Validates the server configuration
    pub fn validate(&self) -> Result<(), HttpServerConfigError> {
        // Basic validation of listen address format
        if !self.listen.contains(':') {
            return Err(HttpServerConfigError::InvalidListenAddress(
                "Address must be in format 'ip:port' or ':port'".to_string(),
            ));
        }
        Ok(())
    }
}
