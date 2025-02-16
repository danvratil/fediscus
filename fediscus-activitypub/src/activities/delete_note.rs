// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use tracing::{info, instrument};
use url::Url;

use crate::apub::DeleteNote;
use crate::FederationData;

use super::ActivityError;

#[async_trait]
impl ActivityHandler for DeleteNote {
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

    #[instrument(name = "receive_delete_note", skip_all, fields(actor=%self.actor, object=%self.object.id))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received note delete from {}", self.actor);

        data.service
            .delete_note(self.object.id.into())
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to delete note"))?;
        Ok(())
    }
}
