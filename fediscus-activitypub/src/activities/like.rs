use activitypub_federation::config::Data;
use activitypub_federation::error::Error as FederationError;
use activitypub_federation::traits::ActivityHandler;
use async_trait::async_trait;
use thiserror::Error;
use tracing::instrument;
use url::Url;

use crate::apub::Like;
use crate::{storage, FederationData};

use super::ActivityError;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LikeError {
    #[error("Activity error {0}")]
    ActivityError(#[from] FederationError),
    #[error("Post error: {0}")]
    NoteError(#[from] storage::NoteError),
}

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
            .await?;
        Ok(())
    }
}
