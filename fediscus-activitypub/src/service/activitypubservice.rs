use crate::activities::{FollowError, LikeError, UndoFollowError};
use crate::storage::{Account, Storage};
use crate::FederationData;
use crate::{apub::Follow, db::Uri};
use activitypub_federation::config::Data;
use async_trait::async_trait;

#[async_trait]
pub trait ActivityPubService: Send + Sync + 'static {
    fn storage(&self) -> &(dyn Storage + Send + Sync + 'static);
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

    async fn like_post(&self, post_uri: Uri) -> Result<(), LikeError>;
    async fn unlike_post(&self, post_uri: Uri) -> Result<(), LikeError>;
}
