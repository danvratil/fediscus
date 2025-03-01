use crate::activities::ActivityError;
use crate::storage::{Account, NoteError, Storage};
use crate::FederationData;
use crate::{apub::Follow, db::Uri};
use activitypub_federation::config::Data;
use async_trait::async_trait;

#[async_trait]
pub trait ActivityPubService: Send + Sync + 'static {
    fn storage(&self) -> &(dyn Storage + Send + Sync + 'static);

    async fn handle_follow_accepted(&self, follow_uri: Uri) -> Result<(), ActivityError>;

    async fn handle_follow_rejected(&self, follow_uri: Uri) -> Result<(), ActivityError>;

    async fn handle_follow_request(
        &self,
        actor: Account,
        object: Account,
        follow: Follow,
        data: &Data<FederationData>,
    ) -> Result<(), ActivityError>;

    async fn handle_follow_undone(
        &self,
        follow_uri: Uri,
        data: &Data<FederationData>,
    ) -> Result<(), ActivityError>;

    async fn like_post(&self, post_uri: Uri) -> Result<(), ActivityError>;

    async fn unlike_post(&self, post_uri: Uri) -> Result<(), ActivityError>;

    async fn delete_note(&self, note_uri: Uri) -> Result<(), NoteError>;
}
