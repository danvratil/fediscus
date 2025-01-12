// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use activitypub_federation::error::Error as FederationError;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use thiserror::Error;
use url::Url;

use crate::apub::Note;

use super::ActivityError;

#[derive(Debug, Error)]
pub enum CreateNoteError {
    #[error("Activity error: {0}")]
    ActivityError(#[from] FederationError),
    #[error("Note error: {0}")]
    NoteError(#[from] crate::storage::NoteError),
}

#[async_trait]
impl ActivityHandler for Note {
    type DataType = crate::FederationData;
    type Error = ActivityError;

    fn id(&self) -> &Url {
        self.id.inner()
    }

    fn actor(&self) -> &Url {
        self.attributed_to.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // TODO
        // 1) verify that the actor is followed by us
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let account = self.attributed_to.dereference(&data).await?;
        let parent_node = match self.in_reply_to {
            Some(id) => Some(
                id.dereference(&data)
                    .await
                    .map_err(CreateNoteError::NoteError)?,
            ),
            None => None,
        };
        let root_id = match parent_node {
            Some(parent) => Some(parent.root_id.unwrap_or(parent.id)),
            None => None,
        };

        data.storage
            .new_post(
                account.id,
                self.id.inner().clone().into(),
                parent_node.map(|p| p.id),
                root_id,
                blog.id,
            )
            .await?;

        Ok(())
    }
}
