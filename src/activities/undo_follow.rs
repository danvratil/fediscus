use activitypub_federation::config::Data;
use activitypub_federation::kinds::activity::UndoType;
use activitypub_federation::traits::{Actor, Object};
use activitypub_federation::{fetch::object_id::ObjectId, traits::ActivityHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use crate::apub::UndoFollow;
use crate::{storage, FederationData};

use super::{ActivityError, Follow};

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
        let actor = self.actor.dereference(data).await?;
        let object = self.object.object.dereference_local(data).await?;
        // retrieve us following the actor who unfollowed us
        let follow_back = data.storage.follow_by_ids(object.id, actor.id).await?;

        // Delete the actor's follow
        let follow = storage::Follow::read_from_id(self.object.id.into(), data).await?;
        storage::Follow::delete(follow, data).await?;
        
        // Let the actor know we've unfollowed them as well
        UndoFollow::send(object, follow_back, actor.shared_inbox_or_inbox(), data).await?;
        storage::Follow::delete(follow_back, data).await?;

        Ok(())
    }
}
