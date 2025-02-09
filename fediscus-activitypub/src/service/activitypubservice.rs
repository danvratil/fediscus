use crate::activities::{FollowError, UndoFollowError};
use crate::storage::{Account, Storage};
use crate::FederationData;
use crate::{apub::Follow, db::Uri};
use activitypub_federation::config::Data;
use async_trait::async_trait;

#[async_trait]
pub trait ActivityPubService: Send + Sync + 'static {
    fn storage(&self) -> &Box<dyn Storage + Send + Sync + 'static>;
    async fn handle_follow_accepted(&self, follow_uri: Uri) -> Result<(), FollowError>;
    async fn handle_follow_rejected(&self, follow_uri: Uri) -> Result<(), FollowError>;
    async fn handle_follow_request(
        &self,
        actor: Account,
        object: Account,
        follow: Follow,
        data: &Data<FederationData>,
    ) -> Result<(), FollowError>;
    async fn handle_follow_undone(
        &self,
        follow_uri: Uri,
        data: &Data<FederationData>,
    ) -> Result<(), UndoFollowError>;
}
