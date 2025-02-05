use crate::activities::{FollowError, UndoFollowError};
use crate::apub::{AcceptFollow, Follow, UndoFollow};
use crate::db::Uri;
use crate::storage::{Account, Storage};
use crate::FederationData;
use activitypub_federation::config::Data;
use activitypub_federation::traits::{Actor, Object};
use async_trait::async_trait;
use tracing::info;

use crate::service::ActivityPubService;

pub struct Service {
    storage: Box<dyn Storage + Send + Sync + 'static>,
}

impl Service {
    pub fn new<S: Storage + Send + Sync + 'static>(storage: S) -> Self {
        Self { storage: Box::new(storage) }
    }
}


#[async_trait]
impl ActivityPubService for Service {

    fn storage(&self) -> &Box<dyn Storage + Send + Sync + 'static> {
        &self.storage
    }

    async fn handle_follow_accepted(&self, follow_uri: Uri) -> Result<(), FollowError> {
        self.storage
            .follow_accepted(&follow_uri)
            .await
            .map_err(Into::into)
    }

    async fn handle_follow_rejected(&self, follow_uri: Uri) -> Result<(), FollowError> {
        self.storage
            .delete_follow_by_uri(&follow_uri)
            .await
            .map_err(Into::into)
    }

    async fn handle_follow_request(&self, actor: Account, object: Account, follow: Follow, data: &Data<FederationData>) -> Result<(), FollowError> {

        // Create a new follow in complete state
        self.storage
            .new_follow(
                actor.id,
                object.id,
                &follow.id.inner().clone().into(),
                false,
            )
            .await?;

        AcceptFollow::send(&object, actor.shared_inbox_or_inbox(), follow, data)
            .await
            .map_err(FollowError::AcceptError)?;
        Follow::send(&object, &actor, data)
            .await?;

        Ok(())
    }

    async fn handle_follow_undone(&self, follow_uri: Uri, data: &Data<FederationData>) -> Result<(), UndoFollowError> {
        // Find the original follow that is being undone
        let follow = self.storage.follow_by_uri(&follow_uri).await?;
        // If we don't have one, we are good.
        let follow = if let Some(follow) = follow {
            self.storage.delete_follow_by_id(follow.id).await?;
            follow
        } else {
            info!("UndoFollow: received Undo for non-existent Follow");
            return Ok(());
        };

        // Actor is the one who unfollowed us, so they  are the `account_id` side of the relationship.
        let actor = match self.storage.account_by_id(follow.account_id).await? {
            Some(actor) => actor,
            None => {
                info!("UndoFollow: actor not found");
                return Ok(());
            }
        };

        // Object is the one who we unfollowed, so they are the `target_account_id` side of the relationship (us, effectively).
        let object = match self.storage.account_by_id(follow.target_account_id).await? {
            Some(object) => object,
            None => {
                info!("UndoFollow: object not found");
                return Ok(());
            }
        };

        // Obtain the inverse follow relationship (us following the original actor), so we can cancel it.
        let follow_back = self.storage.follow_by_ids(follow.target_account_id, follow.account_id).await?;
        if let Some(follow_back) = follow_back {
            // Let the actor know we've unfollowed them as well
            UndoFollow::send(
                &object,
                follow_back
                    .clone()
                    .into_json(data)
                    .await?,
                actor.shared_inbox_or_inbox(),
                data,
            )
            .await?;

            self.storage.delete_follow_by_id(follow_back.id).await?;
        } else {
            info!("UndoFollow: not sending Undo to user we don't follow");
        }

        Ok(())
    }
}
