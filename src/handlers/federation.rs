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
use url::Url;

use crate::objects::DbPerson;
use crate::{AppState, FederationData, activities};

#[derive(Clone, Debug)]
pub struct LocalUser(DbPerson);

impl Deref for LocalUser {
    type Target = DbPerson;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl LocalUser {
    pub fn into_db_person(self) -> DbPerson {
        self.0
    }
}

impl TryFrom<DbPerson> for LocalUser {
    type Error = Error;
    fn try_from(value: DbPerson) -> Result<Self, Self::Error> {
        if value.local {
            Ok(Self(value))
        } else {
            Err(anyhow::anyhow!("Not a local user"))
        }
    }
}

async fn get_local_user(data: &Data<FederationData>) -> Result<LocalUser, Error> {
    let user = sqlx::query!(
        r#"
        SELECT uri FROM accounts WHERE local = 1
        "#,
    )
    .fetch_one(&data.db)
    .await
    .unwrap_or_else(|_| panic!("No local user found"));

    LocalUser::try_from(
        DbPerson::read_from_id(Url::parse(&user.uri)?, &data)
            .await?
            .unwrap_or_else(|| panic!("No local user found")),
    )
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
enum LocalUserAcceptedActivities {
    Follow(activities::Follow),
    Accept(activities::Accept),
    Reject(activities::Reject),
    Undo(activities::Undo),
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
            .into_db_person()
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
    receive_activity::<WithContext<LocalUserAcceptedActivities>, DbPerson, FederationData>(
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
    let person = DbPerson::read_from_name(&name, &data)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if let Some(person) = person {
        Ok(FederationJson(APubOutbox::empty(person.outbox)))
    } else {
        Err(StatusCode::NOT_FOUND)
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
