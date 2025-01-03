use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use activitypub_federation::traits::{Actor, Object};
use async_trait::async_trait;
use thiserror::Error;
use tracing::{info, instrument};
use url::Url;

use crate::apub::{Follow, UndoFollow};
use crate::{storage, FederationData};

use super::{generate_activity_id, ActivityError};

#[derive(Error, Debug)]
pub enum UndoFollowError {
    #[error("Account error: {0}")]
    AccountError(#[from] storage::AccountError),
    #[error("Activity error {0}")]
    ActivityError(#[from] activitypub_federation::error::Error),
    #[error("Follow error: {0}")]
    FollowError(#[from] storage::FollowError),
}

impl UndoFollow {
    #[instrument(skip_all)]
    async fn send(
        actor: &storage::Account,
        follow: Follow,
        inbox: Url,
        data: &Data<FederationData>,
    ) -> Result<(), UndoFollowError> {
        let activity = WithContext::new_default(UndoFollow::new(
            actor.uri.clone().into(),
            follow,
            generate_activity_id(data),
        ));
        queue_activity(&activity, actor, vec![inbox], data)
            .await
            .map_err(UndoFollowError::ActivityError)
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

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        info!("Received UndoFollow from {}", self.actor);
        let actor = self
            .actor
            .dereference(data)
            .await
            .map_err(UndoFollowError::AccountError)?;
        let object = self
            .object
            .object
            .dereference_local(data)
            .await
            .map_err(UndoFollowError::AccountError)?;
        // retrieve us following the actor who unfollowed us
        let follow_back = data
            .storage
            .follow_by_ids(object.id, actor.id)
            .await
            .map_err(UndoFollowError::FollowError)?;

        // Delete the actor's follow
        let follow = storage::Follow::read_from_id(self.object.id.into(), data)
            .await
            .map_err(UndoFollowError::FollowError)?;
        if let Some(follow) = follow {
            storage::Follow::delete(follow, data)
                .await
                .map_err(UndoFollowError::FollowError)?;
        } else {
            info!("UndoFollow: received Undo for non-existent Follow");
        }

        if let Some(follow_back) = follow_back {
            // Let the actor know we've unfollowed them as well
            UndoFollow::send(
                &object,
                follow_back
                    .clone()
                    .into_json(data)
                    .await
                    .map_err(UndoFollowError::FollowError)?,
                actor.shared_inbox_or_inbox(),
                data,
            )
            .await?;
            storage::Follow::delete(follow_back, data)
                .await
                .map_err(UndoFollowError::FollowError)?;
        } else {
            info!("UndoFollow: not sending Undo to user we don't follow");
        }

        Ok(())
    }
}
