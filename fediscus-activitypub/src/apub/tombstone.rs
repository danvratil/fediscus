use activitypub_federation::kinds::object::TombstoneType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    pub r#type: TombstoneType,
    pub id: Url,
}
