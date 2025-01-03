// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use std::ops::Deref;
use std::sync::Arc;

use activitypub_federation::axum::inbox::{ActivityData, receive_activity};
use activitypub_federation::config::Data;
use activitypub_federation::fetch::webfinger::{Webfinger, WebfingerLink};
use activitypub_federation::kinds::collection::OrderedCollectionType;
use activitypub_federation::traits::{ActivityHandler, Object};
use activitypub_federation::{
    FEDERATION_CONTENT_TYPE, axum::json::FederationJson, protocol::context::WithContext,
};
use anyhow::Error;
use axum::extract::{Query, State};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hyperx::header::{Accept, Header, Raw};
use serde::{Deserialize, Serialize};
use tracing::error;
use url::Url;

use crate::apub;
use crate::{storage, AppState, FederationData};

#[derive(Clone, Debug)]
pub struct LocalUser(storage::Account);

impl Deref for LocalUser {
    type Target = storage::Account;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl LocalUser {
    pub fn into_account(self) -> storage::Account {
        self.0
    }
}

impl TryFrom<storage::Account> for LocalUser {
    type Error = Error;
    fn try_from(value: storage::Account) -> Result<Self, Self::Error> {
        if value.local {
            Ok(Self(value))
        } else {
            Err(anyhow::anyhow!("Not a local user"))
        }
    }
}

async fn get_local_user(data: &Data<FederationData>) -> Result<LocalUser, Error> {
    let account = data.storage.get_local_account().await?;
    LocalUser::try_from(account)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
enum LocalUserAcceptedActivities {
    Follow(apub::Follow),
    Accept(apub::AcceptFollow),
    Reject(apub::RejectFollow),
    Undo(apub::UndoFollow),
}

#[axum::debug_handler]
pub async fn get_user(
    header_map: HeaderMap,
    Path(name): Path<String>,
    data: Data<FederationData>,
) -> impl IntoResponse {
    let accept = header_map
        .get("accept")
        .map(|v| Raw::from(v.as_bytes()))
        .ok_or(StatusCode::BAD_REQUEST)?;
    let accept = Accept::parse_header(&accept).map_err(|_| StatusCode::BAD_REQUEST)?;
    if accept.iter().any(|v| v.item == FEDERATION_CONTENT_TYPE) {
        let local_user = get_local_user(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if local_user.username != name {
            return Err(StatusCode::NOT_FOUND);
        }

        let json_user = local_user
            .into_account()
            .into_json(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(FederationJson(WithContext::new_default(json_user)).into_response())
    } else {
        println!("Received {:?}", accept);
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn post_inbox(
    State(_state): State<Arc<AppState>>,
    data: Data<FederationData>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    receive_activity::<WithContext<LocalUserAcceptedActivities>, storage::Account, FederationData>(
        activity_data,
        &data,
    )
    .await
    .map_err(|e| {
        eprintln!("Error receiving activity: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct APubOutbox {
    id: Url,
    r#type: OrderedCollectionType,
    total_items: i32,
    ordered_items: Vec<()>
}

impl APubOutbox {
    pub fn empty(id: Url) -> Self {
        Self {
            id,
            r#type: OrderedCollectionType::OrderedCollection,
            total_items: 0,
            ordered_items: vec![],
        }
    }
}

pub async fn get_outbox(
    Path((name, )): Path<(String,)>,
    data: Data<FederationData>,
) -> impl IntoResponse {
    let local_user = get_local_user(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if name != local_user.username {
        return Err(StatusCode::NOT_FOUND);
    }

    match &local_user.outbox {
        Some(outbox) => {
            let outbox = APubOutbox::empty(outbox.clone().into());
            Ok(FederationJson(WithContext::new_default(outbox)).into_response())
        },
        None => {
            error!("Local account without valid outbox URL");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WebFingerQuery {
    resource: String,
    rel: Option<String>,
}

pub async fn get_webfinger(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<WebFingerQuery>,
    data: Data<FederationData>,
) -> impl IntoResponse {
    if query.resource.starts_with("acct:") {
        let local_user = get_local_user(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if query.resource != format!("acct:{}@{}", local_user.username, local_user.host) {
            return Err(StatusCode::NOT_FOUND);
        }

        let webfinger = Webfinger {
            subject: query.resource,
            aliases: vec![
                Url::parse(&format!(
                    "https://{}/{}",
                    local_user.host, local_user.username
                ))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                Url::parse(&format!(
                    "https://{}/users/{}",
                    local_user.host, local_user.username
                ))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            ],
            links: vec![WebfingerLink {
                rel: Some("self".to_string()),
                kind: Some("application/activity+json".to_string()),
                href: Some(
                    Url::parse(&format!(
                        "https://{}/users/{}",
                        local_user.host, local_user.username
                    ))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                ),
                ..Default::default()
            }],
            ..Default::default()
        };

        Ok(FederationJson(WithContext::new_default(webfinger)).into_response())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
