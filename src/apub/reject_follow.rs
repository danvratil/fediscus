use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::RejectType};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{storage, apub};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectFollow {
    pub actor: ObjectId<storage::Account>,
    pub object: apub::Follow,
    r#type: RejectType,
    pub id: Url,
}

impl RejectFollow {
    pub fn new(
        actor: ObjectId<storage::Account>,
        object: apub::Follow,
        id: Url,
    ) -> Self {
        Self {
            actor,
            object,
            r#type: RejectType::Reject,
            id
        }
    }
}

