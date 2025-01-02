use activitypub_federation::config::{Data, FederationConfig};
use thiserror::Error;

mod accept;
mod follow;
mod reject;
mod undo;

#[derive(Error, Debug)]
pub enum ActivityError {
    #[error("Error handling incoming activity")]
    InboxError(#[from] activitypub_federation::error::Error),
    #[error("Follow activity error")]
    FollowError(#[from] follow::FollowError),
    #[error("Accept activity error")]
    AcceptError(#[from] accept::AcceptError),
    #[error("Unknown error")]
    UnknownError(#[from] anyhow::Error),
    #[error("User error")]
    UserError(#[from] crate::objects::UserError),
}

pub use accept::Accept;
pub use follow::Follow;
pub use reject::Reject;
pub use undo::Undo;
use url::Url;
use uuid::Uuid;

use crate::FederationData;

fn generate_activity_id(data: &Data<FederationData>) -> Url {
    let id = Uuid::new_v4();
    Url::parse(&format!("https://{}/activity/{}", data.config.fediverse_user.instance, id)).unwrap()
}