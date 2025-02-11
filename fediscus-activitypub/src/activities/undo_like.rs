// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use thiserror::Error;
use tracing::{info, instrument};
use url::Url;

use crate::apub::{Like, UndoLike};
use crate::{storage, FederationData};

use super::{generate_activity_id, ActivityError};

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum UndoLikeError {
    #[error("Account error: {0}")]
    AccountError(#[from] storage::AccountError),
    #[error("Activity error {0}")]
    ActivityError(#[from] activitypub_federation::error::Error),
    #[error("Note error: {0}")]
    NoteError(#[from] storage::NoteError),
}

impl UndoLike {
    #[instrument(skip_all)]
    pub async fn send(
        actor: &storage::Account,
        like: Like,
        inbox: Url,
        data: &Data<FederationData>,
    ) -> Result<(), UndoLikeError> {
        let activity = WithContext::new_default(UndoLike::new(
            actor.uri.clone().into(),
            like,
            generate_activity_id(data),
        ));
        queue_activity(&activity, actor, vec![inbox], data)
            .await
            .map_err(UndoLikeError::ActivityError)
    }
}

#[async_trait]
impl ActivityHandler for UndoLike {
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

    #[instrument(name="undo_like_receive", skip_all, fields(actor=%self.actor.inner(), object=%self.object.id()))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received UndoLike from {}", self.actor);

        data.service
            .unlike_post(self.object.id().clone().into())
            .await?;
        Ok(())
    }
}
