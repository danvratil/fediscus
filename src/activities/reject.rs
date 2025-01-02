use activitypub_federation::config::Data;
use activitypub_federation::kinds::activity::RejectType;
use activitypub_federation::{fetch::object_id::ObjectId, traits::ActivityHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

use crate::objects::DbPerson;
use crate::FederationData;

use super::{generate_activity_id, ActivityError, Follow};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reject {
    actor: ObjectId<DbPerson>,
    object: Follow,
    r#type: RejectType,
    id: Url,
}

impl Reject {
    pub fn new(actor: ObjectId<DbPerson>, object: Follow, data: &Data<FederationData>) -> Self {
        Self {
            actor,
            object,
            r#type: RejectType::Reject,
            id: generate_activity_id(data),
        }
    }
}

#[async_trait]
impl ActivityHandler for Reject {
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
        info!("Received rejection to our follow request from {:?}", self.actor);
        Ok(())
    }
}
