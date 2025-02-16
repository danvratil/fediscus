// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::{ActivityHandler, Actor, Object};
use async_trait::async_trait;
use tracing::{info, instrument};
use url::Url;

use crate::apub::Follow;
use crate::{storage, FederationData};

use super::{generate_activity_id, ActivityError};

impl Follow {
    #[instrument(skip_all)]
    pub async fn send(
        actor: &storage::Account,
        object: &storage::Account,
        data: &Data<FederationData>,
    ) -> Result<(), ActivityError> {
        // Send a follow activity back - we just swap the object and actor
        let follow = WithContext::new_default(Follow::new(
            actor.uri.clone().into(),
            object.uri.clone().into(),
            generate_activity_id(data)?.into(),
        ));

        storage::Follow::from_json(follow.inner().clone(), data)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to save follow activity"))?;

        queue_activity(&follow, actor, vec![object.shared_inbox_or_inbox()], data)
            .await
            .map_err(|e| ActivityError::federation(e, "Failed to queue follow activity"))
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
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[instrument(name = "receive_follow", skip_all, fields(actor=%self.actor, object=%self.object))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received follow request from {}", self.actor);

        let actor = self.actor.dereference(data).await.map_err(|e| {
            ActivityError::storage(e, format!("Failed to dereference actor {}", self.actor))
        })?;

        let object = self.object.dereference(data).await.map_err(|e| {
            ActivityError::storage(e, format!("Failed to dereference object {}", self.object))
        })?;

        data.service
            .handle_follow_request(actor, object, self, data)
            .await
            .map_err(|e| ActivityError::processing(e, "Failed to handle follow request"))?;

        Ok(())
    }
}
