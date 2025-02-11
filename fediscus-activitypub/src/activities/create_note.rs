// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use activitypub_federation::error::Error as FederationError;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::kinds::activity::CreateType;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, instrument};
use url::Url;

use crate::apub::Note as APubNote;
use crate::storage::{self, Account, Note};

use super::ActivityError;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum CreateNoteError {
    #[error("Activity error: {0}")]
    ActivityError(#[from] FederationError),
    #[error("Note error: {0}")]
    NoteError(#[from] crate::storage::NoteError),
    #[error("Blog error: {0}")]
    BlogError(#[from] crate::storage::BlogError),
    #[error("Invalid note content")]
    ContentError,
}

impl APubNote {
    #[instrument(name="create_note_top_level", skip_all, fields(actor=%account.uri, object=%self.id.inner()))]
    async fn handle_top_level_note(
        &self,
        data: &Data<crate::FederationData>,
        account: &Account,
    ) -> Result<(), ActivityError> {
        // A top-level note must contain the #fediscus tag, otherwise it's not interesting to us
        if !self.has_tag() {
            debug!("Note does not have #fediscus tag, ignoring");
            return Ok(());
        }

        let urls = self
            .get_links()
            .map_err(|_| CreateNoteError::ContentError)?;
        // And it must have at least one link to the blog post, duh!
        if urls.is_empty() {
            debug!("Note does not have any links, ignoring");
            return Ok(());
        }

        // We are only interested in the first URL, which should point to the blog post. Obviously any
        // of the URLs could point to the blog post, but do we have an oracle to tell us which one?
        let blog_url = &urls[0];

        let blog = data
            .service
            .storage()
            .new_blog(blog_url)
            .await
            .map_err(|e| ActivityError::CreateNoteError(e.into()))?;

        data.service
            .storage()
            .new_post(
                account.id,
                self.id.inner().clone().into(),
                None,
                None,
                blog.id,
            )
            .await
            .map_err(|e| ActivityError::CreateNoteError(e.into()))?;

        Ok(())
    }

    #[instrument(name="create_note_reply", skip_all, fields(actor=%account.uri, object=%self.id.inner()))]
    async fn handle_reply_note(
        &self,
        data: &Data<crate::FederationData>,
        account: &Account,
        parent_note: &Note,
    ) -> Result<(), ActivityError> {
        data.service
            .storage()
            .new_post(
                account.id,
                self.id.inner().clone().into(),
                Some(parent_note.id),
                parent_note.root_id.or(Some(parent_note.id)),
                parent_note.blog_id,
            )
            .await
            .map_err(|e| ActivityError::CreateNoteError(e.into()))?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNote {
    r#type: CreateType,
    actor: ObjectId<storage::Account>,
    id: ObjectId<storage::Note>,
    object: APubNote,
}

#[async_trait]
impl ActivityHandler for CreateNote {
    type DataType = crate::FederationData;
    type Error = ActivityError;

    fn id(&self) -> &Url {
        self.id.inner()
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // TODO
        // 1) verify that the actor is followed by us
        Ok(())
    }

    #[instrument(name="create_note_receive", skip_all, fields(actor=%self.actor.inner(), object=%self.object.id.inner()))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received note from {}", self.actor.inner());
        let account = self.object.attributed_to.dereference(data).await?;
        // Figure out whether this note is a reply or top-level note. Reply must be a reply
        // to a known top-level note.
        let parent_note = match &self.object.in_reply_to {
            // Only try to dereference the reply locally - if it's not found it means that this
            // note is a reply to a note that we did not consider interesting, so we can just ignore it.
            Some(id) => {
                match data
                    .app_data()
                    .service
                    .storage()
                    .post_by_uri(&id.inner().clone().into())
                    .await
                {
                    Ok(Some(note)) => Some(note), // Parent exists locally
                    Ok(None) => None,             // Parent doesn't exist locally - ignor this reply
                    Err(err) => return Err(ActivityError::CreateNoteError(err.into())), // Something went wrong
                }
            }
            None => None, // OK, this is a top-level note
        };

        match parent_note {
            Some(parent_note) => {
                self.object
                    .handle_reply_note(data, &account, &parent_note)
                    .await
            }
            None => self.object.handle_top_level_note(data, &account).await,
        }
    }
}
