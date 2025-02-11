// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use thiserror::Error;

use crate::storage;

mod accept_follow;
mod create_note;
mod follow;
mod like;
mod reject_follow;
mod undo_follow;
mod undo_like;
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ActivityError {
    #[error("Account error: {0}")]
    AccountError(#[from] storage::AccountError),
    #[error("Error handling incoming activity {0}")]
    InboxError(#[from] activitypub_federation::error::Error),
    #[error("Follow activity error {0}")]
    FollowError(#[from] follow::FollowError),
    #[error("Accept follow activity error {0}")]
    AcceptError(#[from] accept_follow::AcceptError),
    #[error("Reject follow activity error {0}")]
    RejectError(#[from] reject_follow::RejectError),
    #[error("Undo follow activity error {0}")]
    UndoError(#[from] undo_follow::UndoFollowError),
    #[error("Create note activity error {0}")]
    NoteError(#[from] create_note::CreateNoteError),
    #[error("Like activity error {0}")]
    LikeError(#[from] like::LikeError),
    #[error("Unknown error {0}")]
    UnknownError(#[from] anyhow::Error),
}

use url::Url;
use uuid::Uuid;

pub use create_note::CreateNote;
pub use follow::FollowError;
pub use like::LikeError;
pub use undo_follow::UndoFollowError;

use crate::FederationData;

fn generate_activity_id(data: &Data<FederationData>) -> Url {
    let id = Uuid::new_v4();
    Url::parse(&format!(
        "https://{}/activity/{}",
        data.config.fediverse_user.host, id
    ))
    .unwrap()
}
