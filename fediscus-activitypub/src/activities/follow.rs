// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use std::fmt::Debug;

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::error::Error as FederationError;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::{ActivityHandler, Actor, Object};
use async_trait::async_trait;
use thiserror::Error;
use tracing::{info, instrument, warn};
use url::Url;

use crate::apub::Follow;
use crate::{storage, FederationData};

use super::accept_follow::AcceptError;
use super::{generate_activity_id, ActivityError};

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum FollowError {
    #[error("Account error: {0}")]
    AccountError(#[from] storage::AccountError),
    #[error("Activity error {0}")]
    ActivityError(#[from] FederationError),
    #[error("Follow error: {0}")]
    FollowError(#[from] storage::FollowError),
    #[error("Failed to accept follow request: {0}")]
    AcceptError(#[from] AcceptError),
}

impl Follow {
    #[instrument(skip_all)]
    pub async fn send(
        actor: &storage::Account,
        object: &storage::Account,
        data: &Data<FederationData>,
    ) -> Result<(), FollowError> {
        // Send a follow activity back - we just swap the object and actor
        let follow = WithContext::new_default(Follow::new(
            actor.uri.clone().into(),
            object.uri.clone().into(),
            generate_activity_id(data).into(),
        ));

        storage::Follow::from_json(follow.inner().clone(), data)
            .await
            .map_err(FollowError::FollowError)?;

        queue_activity(&follow, actor, vec![object.shared_inbox_or_inbox()], data)
            .await
            .map_err(FollowError::ActivityError)
    }
}

#[async_trait]
impl ActivityHandler for Follow {
    type DataType = FederationData;
    type Error = ActivityError;

    fn id(&self) -> &Url {
        self.id.inner()
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    /// Verifies that the object that is being followed is a local account.
    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let object = self
            .object
            .dereference(data)
            .await
            .map_err(FollowError::AccountError)?;
        if !object.local {
            warn!("Received follow request for non-local account");
            return Err(FollowError::AccountError(storage::AccountError::NotFound).into());
        }

        Ok(())
    }

    #[instrument(name="follow_receive", skip_all, fields(actor=%self.actor.inner(), object=%self.object.inner()))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received follow request from {}", self.actor.inner());

        let actor = self.actor.dereference(data).await?;
        let object = self.object.dereference_local(data).await?;

        data.service.handle_follow_request(actor, object, self.clone(), data).await?;
        Ok(())
    }
}
