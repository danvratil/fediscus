use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::FollowType};
use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    /// Who sent as the follow request
    pub actor: ObjectId<storage::Account>,
    // The target object that the actor wants to follow
    // Only our LocalUser can be followed for now.
    pub object: ObjectId<storage::Account>,
    r#type: FollowType,
    /// ID of the follow activity
    pub id: ObjectId<storage::Follow>,
}

impl Follow {
    pub fn new(
        actor: ObjectId<storage::Account>,
        object: ObjectId<storage::Account>,
        id: ObjectId<storage::Follow>,
    ) -> Self {
        Self {
            actor,
            object,
            r#type: FollowType::Follow,
            id,
        }
    }
}

