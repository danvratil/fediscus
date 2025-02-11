use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::LikeType};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::storage;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    pub actor: ObjectId<storage::Account>,
    pub object: ObjectId<storage::Note>,
    r#type: LikeType,
    pub id: Url,
}
