use activitypub_federation::config::Data;
use thiserror::Error;

mod accept_follow;
mod follow;
mod reject_follow;
mod undo_follow;

#[derive(Error, Debug)]
pub enum ActivityError {
    #[error("Error handling incoming activity {0}")]
    InboxError(#[from] activitypub_federation::error::Error),
    #[error("Follow activity error {0}")]
    FollowError(#[from] follow::FollowError),
    #[error("Accept activity error {0}")]
    AcceptError(#[from] accept_follow::AcceptError),
    #[error("Reject activity error {0}")]
    RejectError(#[from] reject_follow::RejectError),
    #[error("Unknown error {0}")]
    UnknownError(#[from] anyhow::Error),
}

use url::Url;
use uuid::Uuid;

use crate::FederationData;

fn generate_activity_id(data: &Data<FederationData>) -> Url {
    let id = Uuid::new_v4();
    Url::parse(&format!("https://{}/activity/{}", data.config.fediverse_user.host, id)).unwrap()
}