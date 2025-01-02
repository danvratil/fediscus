use activitypub_federation::config::Data;
use activitypub_federation::kinds::activity::UndoType;
use activitypub_federation::{fetch::object_id::ObjectId, traits::ActivityHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use crate::FederationData;
use crate::objects::DbPerson;

use super::{ActivityError, Follow};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Undo {
    actor: ObjectId<DbPerson>,
    object: Follow,
    r#type: UndoType,
    id: Url,
}

impl Undo {
    pub fn new(actor: ObjectId<DbPerson>, object: Follow, id: Url) -> Self {
        Self {
            actor,
            object,
            r#type: UndoType::Undo,
            id,
        }
    }
}

#[async_trait]
impl ActivityHandler for Undo {
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

    async fn receive(self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {



        debug!("Received unfollow request from {:?}", self.actor);
        debug!("Follow obj {:?}", self);

        Ok(())
    }
}
