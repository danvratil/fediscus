use activitypub_federation::config::Data;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use tracing::instrument;
use url::Url;

use crate::apub::Like;
use crate::FederationData;

use super::ActivityError;

#[async_trait]
impl ActivityHandler for Like {
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

    #[instrument(name="like_receive", skip_all, fields(actor=%self.actor.inner(), object=%self.object.inner()))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        data.service
            .like_post(self.object.inner().clone().into())
            .await
            .map_err(|e| ActivityError::processing(e, "Failed to process like"))?;
        Ok(())
    }
}
