use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::AcceptType, traits::ActivityHandler};
use activitypub_federation::config::Data;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::info;
use url::Url;

use crate::objects::DbPerson;
use crate::FederationData;

use super::{generate_activity_id, ActivityError, Follow};

#[derive(Error, Debug)]
pub enum AcceptError {
    #[error("SQL error")]
    SqlError(#[from] sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    actor: ObjectId<DbPerson>,
    object: Follow,
    r#type: AcceptType,
    id: Url,
}

impl Accept {
    pub fn new(actor: ObjectId<DbPerson>, object: Follow, data: &Data<FederationData>) -> Self {
        Self {
            actor,
            object,
            r#type: AcceptType::Accept,
            id: generate_activity_id(data),
        }
    }
}

#[async_trait]
impl ActivityHandler for Accept {
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
        info!("User {:?} accepted our follow request", self.actor);

        let uid = self.actor.inner().as_str();
        sqlx::query!(
            "UPDATE fediverse_account SET followed = true WHERE uid = ?",
            uid
        ).execute(&data.db).await.map_err(AcceptError::SqlError)?;
        Ok(())
    }
}