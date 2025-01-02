use std::fmt::Debug;

use activitypub_federation::activity_queue::queue_activity;
use activitypub_federation::config::Data;
use activitypub_federation::error::Error as FederationError;
use activitypub_federation::kinds::activity::FollowType;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::{Actor, Object};
use activitypub_federation::{fetch::object_id::ObjectId, traits::ActivityHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, instrument, warn};
use url::Url;

use crate::objects::UserError;
use crate::{FederationData, objects::DbPerson};

use super::{generate_activity_id, Accept, ActivityError, Reject};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    /// Who sent as the follow request
    pub actor: ObjectId<DbPerson>,
    // The target object that the actor wants to follow
    // Only our LocalUser can be followed for now.
    pub object: ObjectId<DbPerson>,
    r#type: FollowType,
    id: Url,
}

impl Follow {
    pub fn new(actor: ObjectId<DbPerson>, object: ObjectId<DbPerson>, data: &Data<FederationData>) -> Self {
        Self {
            actor,
            object,
            r#type: FollowType::Follow,
            id: generate_activity_id(data),
        }
    }
}

#[derive(Error, Debug)]
pub enum FollowError {
    #[error("Couldn't parse ID as URL")]
    InvalidId(#[from] url::ParseError),
    #[error("Activity error")]
    ActivityError(#[from] FederationError),
    #[error("SQL error")]
    SqlError(#[from] sqlx::Error),
    #[error("Failed to dereference actor")]
    DereferenceError(#[from] anyhow::Error),
    #[error("Error handling user")]
    UserError(#[from] UserError),
}

impl Follow {
    #[instrument(skip_all)]
    async fn reject_follow_request(&self, data: &Data<FederationData>) -> Result<(), FollowError> {
        let reject = WithContext::new_default(Reject::new(self.object.clone(), self.clone(), data));
        let actor = self.object.dereference_local(data).await?;
        let object = self.actor.dereference(data).await?;
        queue_activity(&reject, &actor, vec![object.shared_inbox_or_inbox()], data)
            .await
            .map_err(FollowError::ActivityError)
    }

    #[instrument(skip_all)]
    async fn accept_follow_request(&self, data: &Data<FederationData>) -> Result<(), FollowError> {
        let accept = WithContext::new_default(Accept::new(self.object.clone(), self.clone(), data));
        let actor = self.object.dereference_local(data).await?;
        let object = self.actor.dereference(data).await?;
        queue_activity(&accept, &actor, vec![object.shared_inbox_or_inbox()], data)
            .await
            .map_err(FollowError::ActivityError)
    }

    #[instrument(skip_all)]
    async fn follow_actor(&self, data: &Data<FederationData>) -> Result<(), FollowError> {
        // Send a follow activity back - we just swap the object and actor
        let follow = WithContext::new_default(Follow::new(self.object.clone(), self.actor.clone(), data));
        let actor = self.object.dereference_local(data).await?;
        let object = self.actor.dereference(data).await?;
        queue_activity(&follow, &actor, vec![object.shared_inbox_or_inbox()], data)
            .await
            .map_err(FollowError::ActivityError)
    }

    #[instrument(skip_all)]
    async fn create_new_user(&self, data: &Data<FederationData>) -> Result<(), FollowError> {
        let actor = self.actor.dereference_forced(data).await?;
        let inbox = actor.inbox.as_str();
        let outbox = actor.outbox.as_str();
        let uri = actor.id.inner().as_str();
        let query = sqlx::query!(
            "
            INSERT OR REPLACE INTO accounts \
            (username, host, uri, inbox, outbox, public_key, local) \
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
            actor.username,
            actor.host,
            uri,
            inbox,
            outbox,
            actor.public_key,
            false,
        );
        query
            .execute(&data.db)
            .await
            .map_err(FollowError::SqlError)?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for Follow {
    type DataType = FederationData;
    type Error = ActivityError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // TODO: We could verify whether the activity is for us, more specifically
        // whether the object is our local user.
        //debug!("Verification of Follow activity not implemented");
        Ok(())
    }

    #[instrument(name="follow_receive", skip_all, fields(actor=%self.actor.inner(), object=%self.object.inner()))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let actor = DbPerson::read_from_id(self.actor.inner().clone(), data)
            .await
            .map_err(|e| {
                warn!("Erorr while retrieving actor from local DB: {:?}", e);
                ActivityError::FollowError(FollowError::UserError(e))
            })?;

        match actor {
            // We have an existing record of the person
            Some(actor) => {
                // We accept the follow request, even if its a duplicate
                self.accept_follow_request(data).await?;
                if !actor.followed {
                    // If we don't follow the actor yet, follow them back
                    self.follow_actor(data).await
                } else {
                    Ok(())
                }
            },
            // We never heard of this person before
            None => {
                self.create_new_user(data).await?;
                self.accept_follow_request(data).await?;
                self.follow_actor(data).await
            }
        }
        .map_err(ActivityError::FollowError)
    }
}
