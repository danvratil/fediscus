// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::kinds::activity::CreateType;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use url::Url;

use crate::apub::Note as APubNote;
use crate::storage::{self, Account, Note};

use super::ActivityError;

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
            .map_err(|_| ActivityError::invalid_data("Note does not have any links"))?;
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
            .map_err(|e| ActivityError::storage(e, "Failed to create new blog"))?;

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
            .map_err(|e| ActivityError::storage(e, "Failed to create new post"))?;

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
            .map_err(|e| ActivityError::storage(e, "Failed to create new reply post"))?;
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

impl CreateNote {
    /// Attempts to find the parent note if this is a reply
    async fn find_parent_note(
        &self,
        data: &Data<<Self as ActivityHandler>::DataType>,
    ) -> Result<Option<Note>, ActivityError> {
        match &self.object.in_reply_to {
            Some(id) => data
                .app_data()
                .service
                .storage()
                .post_by_uri(&id.inner().clone().into())
                .await
                .map_err(|e| ActivityError::storage(e, "Failed to find parent note")),
            None => Ok(None),
        }
    }

    /// Processes the note based on whether it's a reply or top-level note
    async fn process_note(
        self,
        data: &Data<<Self as ActivityHandler>::DataType>,
        account: &Account,
        parent_note: Option<Note>,
    ) -> Result<(), ActivityError> {
        match parent_note {
            Some(parent) => self.object.handle_reply_note(data, account, &parent).await,
            None => self.object.handle_top_level_note(data, account).await,
        }
    }
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

    /// Handles the receipt of a new note activity
    ///
    /// This function processes incoming notes, handling both top-level notes
    /// and replies appropriately.
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received note from {}", self.actor.inner());
        let account = self
            .object
            .attributed_to
            .dereference(data)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to dereference actor"))?;

        let parent_note = self.find_parent_note(data).await?;
        self.process_note(data, &account, parent_note).await
    }
}
