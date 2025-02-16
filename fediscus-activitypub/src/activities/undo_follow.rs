// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use tracing::{info, instrument};
use url::Url;

use crate::apub::{Follow, UndoFollow};
use crate::{storage, FederationData};

use super::{generate_activity_id, ActivityError};

impl UndoFollow {
    #[instrument(skip_all)]
    pub async fn send(
        actor: &storage::Account,
        follow: Follow,
        inbox: Url,
        data: &Data<FederationData>,
    ) -> Result<(), ActivityError> {
        let activity = WithContext::new_default(UndoFollow::new(
            actor.uri.clone().into(),
            follow,
            generate_activity_id(data)?,
        ));
        queue_activity(&activity, actor, vec![inbox], data)
            .await
            .map_err(|e| ActivityError::processing(e, "Failed to send undo follow"))
    }
}

#[async_trait]
impl ActivityHandler for UndoFollow {
    type DataType = FederationData;
    type Error = ActivityError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[instrument(name = "receive_undo_follow", skip_all, fields(actor=%self.actor))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received undo follow from {}", self.actor);

        data.service
            .handle_follow_undone(self.object.id.inner().clone().into(), data)
            .await
            .map_err(|e| ActivityError::processing(e, "Failed to handle undo follow"))?;

        Ok(())
    }
}
