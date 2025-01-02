use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::AcceptType};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{storage, apub};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollow {
    pub actor: ObjectId<storage::Account>,
    pub object: apub::Follow,
    r#type: AcceptType,
    pub id: Url,
}

impl AcceptFollow {
    pub fn new(
        actor: ObjectId<storage::Account>,
        object: apub::Follow,
        id: Url,
    ) -> Self {
        Self {
            actor,
            object,
            r#type: AcceptType::Accept,
            id
        }
    }
}
