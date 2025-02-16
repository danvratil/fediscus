// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

pub use create_note::CreateNote;

use crate::{storage, FederationData};

mod accept_follow;
mod create_note;
mod delete_note;
mod follow;
mod like;
mod reject_follow;
mod undo_follow;
mod undo_like;

#[derive(Error, Debug)]
pub enum ActivityError {
    #[error("Failed to process activity: {context}")]
    Processing {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },

    #[error("Activity verification failed: {context}")]
    Verification {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },

    #[error("Storage operation failed: {context}")]
    Storage {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },

    #[error("Federation protocol error: {context}")]
    Federation {
        #[source]
        source: activitypub_federation::error::Error,
        context: String,
    },

    #[error("Invalid activity data: {context}")]
    InvalidData { context: String },
}

impl ActivityError {
    pub fn processing<E>(error: E, context: impl Into<String>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Processing {
            source: Box::new(error),
            context: context.into(),
        }
    }

    pub fn verification<E>(error: E, context: impl Into<String>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Verification {
            source: Box::new(error),
            context: context.into(),
        }
    }

    pub fn storage<E>(error: E, context: impl Into<String>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Storage {
            source: Box::new(error),
            context: context.into(),
        }
    }

    pub fn federation(
        error: activitypub_federation::error::Error,
        context: impl Into<String>,
    ) -> Self {
        Self::Federation {
            source: error,
            context: context.into(),
        }
    }

    pub fn invalid_data(context: impl Into<String>) -> Self {
        Self::InvalidData {
            context: context.into(),
        }
    }
}

impl From<activitypub_federation::error::Error> for ActivityError {
    fn from(e: activitypub_federation::error::Error) -> Self {
        ActivityError::Federation {
            source: e,
            context: "Federation error".to_string(),
        }
    }
}

impl From<storage::AccountError> for ActivityError {
    fn from(e: storage::AccountError) -> Self {
        ActivityError::Storage {
            source: Box::new(e),
            context: "Account storage error".to_string(),
        }
    }
}

fn generate_activity_id(data: &Data<FederationData>) -> Result<Url, ActivityError> {
    let id = Uuid::new_v4();
    Url::parse(&format!(
        "https://{}/activity/{}",
        data.config.fediverse_user.host, id
    ))
    .map_err(|e| ActivityError::invalid_data(format!("Failed to generate activity ID: {}", e)))
}
