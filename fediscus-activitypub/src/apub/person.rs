// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::{
    fetch::object_id::ObjectId, kinds::actor::PersonType, protocol::public_key::PublicKey,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::storage;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub kind: PersonType,
    pub preferred_username: String,
    pub id: ObjectId<storage::Account>,
    pub inbox: Url,
    pub outbox: Option<Url>,
    pub shared_inbox: Option<Url>,
    pub public_key: PublicKey,
}
