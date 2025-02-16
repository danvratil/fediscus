use crate::activities::ActivityError;
use crate::apub::{AcceptFollow, Follow, UndoFollow};
use crate::db::Uri;
use crate::storage::{Account, NoteError, Storage};
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
        Self {
            storage: Box::new(storage),
        }
    }
}

#[async_trait]
impl ActivityPubService for Service {
    fn storage(&self) -> &(dyn Storage + Send + Sync + 'static) {
        self.storage.as_ref()
    }

    async fn handle_follow_accepted(&self, follow_uri: Uri) -> Result<(), ActivityError> {
        self.storage
            .follow_accepted(&follow_uri)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to accept follow"))
    }

    async fn handle_follow_rejected(&self, follow_uri: Uri) -> Result<(), ActivityError> {
        self.storage
            .delete_follow_by_uri(&follow_uri)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to reject follow"))
    }

    async fn handle_follow_request(
        &self,
        actor: Account,
        object: Account,
        follow: Follow,
        data: &Data<FederationData>,
    ) -> Result<(), ActivityError> {
        // Create a new follow in complete state
        self.storage
            .new_follow(
                actor.id,
                object.id,
                &follow.id.inner().clone().into(),
                false,
            )
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to create new follow"))?;

        AcceptFollow::send(&object, actor.shared_inbox_or_inbox(), follow, data).await?;

        Follow::send(&object, &actor, data).await?;

        Ok(())
    }

    async fn handle_follow_undone(
        &self,
        follow_uri: Uri,
        data: &Data<FederationData>,
    ) -> Result<(), ActivityError> {
        // Find the original follow that is being undone
        let follow = self
            .storage
            .follow_by_uri(&follow_uri)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to find follow"))?;

        // If we don't have one, we are good.
        let follow = if let Some(follow) = follow {
            self.storage
                .delete_follow_by_id(follow.id)
                .await
                .map_err(|e| ActivityError::storage(e, "Failed to delete follow"))?;
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
        let follow_back = self
            .storage
            .follow_by_ids(follow.target_account_id, follow.account_id)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to find follow back"))?;
        if let Some(follow_back) = follow_back {
            // Let the actor know we've unfollowed them as well
            UndoFollow::send(
                &object,
                follow_back.clone().into_json(data).await.map_err(|e| {
                    ActivityError::storage(e, "Failed to convert follow back to json")
                })?,
                actor.shared_inbox_or_inbox(),
                data,
            )
            .await?;

            self.storage
                .delete_follow_by_id(follow_back.id)
                .await
                .map_err(|e| ActivityError::storage(e, "Failed to delete follow back"))?;
        } else {
            info!("UndoFollow: not sending Undo to user we don't follow");
        }

        Ok(())
    }

    async fn like_post(&self, post_uri: Uri) -> Result<(), ActivityError> {
        self.storage
            .like_post(&post_uri)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to like post"))
    }

    async fn unlike_post(&self, post_uri: Uri) -> Result<(), ActivityError> {
        self.storage
            .unlike_post(&post_uri)
            .await
            .map_err(|e| ActivityError::storage(e, "Failed to unlike post"))
    }

    async fn delete_note(&self, note_uri: Uri) -> Result<(), NoteError> {
        let post = self.storage.post_by_uri(&note_uri).await?;
        if let Some(post) = post {
            self.storage.delete_post_by_id(post.id).await?;
        } else {
            info!("DeleteNote: note not found");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apub::Follow;
    use crate::storage::AccountStorage;
    use crate::testing;
    use crate::FederationData;
    use activitypub_federation::config::FederationConfig;
    use activitypub_federation::fetch::object_id::ObjectId;
    use std::sync::Arc;
    use url::Url;

    #[tokio::test]
    async fn test_handle_follow_request() {
        let storage = testing::MemoryStorage::new("example.com");

        let local_account = storage.account_by_id(1.into()).await.unwrap().unwrap();

        let actor = testing::create_person("testuser", "example.com");
        let actor_account = storage.new_account(&actor).await.unwrap();

        let follow = Follow::new(
            ObjectId::<Account>::parse(actor_account.uri.as_str()).unwrap(),
            ObjectId::<Account>::parse(local_account.uri.as_str()).unwrap(),
            Url::parse("http://example.com/follow/1").unwrap().into(),
        );

        let service = Box::new(Service::new(storage));
        let data = FederationData {
            config: crate::Config::load().unwrap(),
            service: Arc::new(service),
        };
        let data = FederationConfig::builder()
            .app_data(data)
            .debug(true)
            .domain("example.com")
            .build()
            .await
            .unwrap();

        data.service
            .handle_follow_request(
                actor_account.clone(),
                local_account.clone(),
                follow.clone(),
                &data.to_request_data(),
            )
            .await
            .expect("Handling follow request failed");

        // Handling the follow request should leave us with a existing follow relationship
        // in non-pending (confirmed) state...
        let created_follow = data
            .service
            .storage()
            .follow_by_uri(&follow.id.inner().clone().into())
            .await
            .expect("Retrieving Follow failed")
            .expect("Follow not found");
        assert_eq!(created_follow.account_id, actor_account.id);
        assert_eq!(created_follow.target_account_id, local_account.id);
        assert!(!created_follow.pending);

        // And a reverse follow relationship in pending state (waiting for the original actor to approve our follow-back)
        let reverse_follow = data
            .service
            .storage()
            .follow_by_ids(local_account.id, actor_account.id)
            .await
            .expect("Retrieving reverse follow failed")
            .expect("Reverse follow not found");
        assert_eq!(reverse_follow.account_id, local_account.id);
        assert_eq!(reverse_follow.target_account_id, actor_account.id);
        assert!(reverse_follow.pending);
    }
}
