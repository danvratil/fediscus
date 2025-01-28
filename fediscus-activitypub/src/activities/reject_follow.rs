// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use activitypub_federation::error::Error as FederationError;
use async_trait::async_trait;
use thiserror::Error;
use tracing::{info, instrument};
use url::Url;

use crate::apub::{RejectFollow, Follow};
use crate::{storage, FederationData};

use super::{generate_activity_id, ActivityError};

#[derive(Error, Debug)]
pub enum RejectError {
    #[error("Error handling follow: {0}")]
    FollowError(#[from] storage::FollowError),
    #[error("Failed to send RejectFollow: {0}")]
    SendError(#[from] FederationError),
}

impl RejectFollow {
    #[instrument(skip_all)]
    pub async fn send(
        actor: &storage::Account,
        inbox: Url,
        object: Follow,
        data: &Data<FederationData>,
    ) -> Result<(), RejectError> {
        let accept = WithContext::new_default(RejectFollow::new(
            actor.uri.clone().into(),
            object,
            generate_activity_id(data),
        ));
        queue_activity(&accept, actor, vec![inbox], data)
            .await
            .map_err(RejectError::SendError)
    }
}

#[async_trait]
impl ActivityHandler for RejectFollow {
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
            .map_err(RejectError::FollowError)?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("User {:?} rejected our follow request", self.actor);

        let request = self
            .object
            .id
            .dereference(data)
            .await
            .map_err(RejectError::FollowError)?;

        data.storage
            .delete_follow_by_uri(&request.uri)
            .await
            .map_err(RejectError::FollowError)?;

        Ok(())
    }
}
