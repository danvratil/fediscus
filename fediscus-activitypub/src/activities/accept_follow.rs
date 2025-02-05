// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::error::Error as FederationError;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use thiserror::Error;
use tracing::{info, instrument};
use url::Url;

use crate::apub::{AcceptFollow, Follow};
use crate::{storage, FederationData};

use super::{generate_activity_id, ActivityError};

#[derive(Error, Debug)]
pub enum AcceptError {
    #[error("Error creating follow: {0}")]
    FollowError(#[from] storage::FollowError),
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("Received acceptance of an unknown follow request")]
    UnknownActivity,
    #[error("Failed to send AcceptFollow: {0}")]
    SendError(#[from] FederationError),
}

impl AcceptFollow {
    #[instrument(skip_all)]
    pub async fn send(
        actor: &storage::Account,
        inbox: Url,
        object: Follow,
        data: &Data<FederationData>,
    ) -> Result<(), AcceptError> {
        let accept = WithContext::new_default(AcceptFollow::new(
            actor.uri.clone().into(),
            object,
            generate_activity_id(data),
        ));
        queue_activity(&accept, actor, vec![inbox], data)
            .await
            .map_err(AcceptError::SendError)
    }
}

#[async_trait]
impl ActivityHandler for AcceptFollow {
    type DataType = FederationData;
    type Error = ActivityError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        self.object
            .id
            .dereference_local(data)
            .await
            .map_err(AcceptError::FollowError)?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("User {:?} accepted our follow request", self.actor);

        let uri = self.object.id;
        data.service
            .handle_follow_accepted(uri.inner().clone().into())
            .await?;
        Ok(())
    }
}
