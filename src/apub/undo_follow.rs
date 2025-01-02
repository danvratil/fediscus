use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::UndoType};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{storage, apub};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollow {
    pub actor: ObjectId<storage::Account>,
    pub object: apub::Follow,
    r#type: UndoType,
    pub id: Url,
}

impl UndoFollow {
    pub fn new(
        actor: ObjectId<storage::Account>,
        object: apub::Follow,
        id: Url,
    ) -> Self {
        Self {
            actor,
            object,
            r#type: UndoType::Undo,
            id
        }
    }
}

