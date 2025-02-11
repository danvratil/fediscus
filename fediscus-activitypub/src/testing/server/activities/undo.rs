use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::kinds::activity::UndoType;
use activitypub_federation::traits::ActivityHandler;
use activitypub_federation::config::Data;
use url::Url;
use serde::{Deserialize, Serialize};

use crate::testing::server::error::Error;
use crate::testing::server::objects::DbUser;
use crate::testing::server::activities::Follow;
use crate::testing::server::DatabaseHandle;


#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Undo {
    pub(crate) actor: ObjectId<DbUser>,
    pub(crate) object: Follow,
    #[serde(rename = "type")]
    kind: UndoType,
    id: Url
}

impl Undo {
    pub fn new(actor: ObjectId<DbUser>, object: Follow, id: Url) -> Undo {
        Undo {
            actor,
            object,
            kind: Default::default(),
            id
        }
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Undo {
    type DataType = DatabaseHandle;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let mut users = data.users.lock().unwrap();
        let local_user = users.first_mut().unwrap();
        local_user.followers.retain(|f| f != self.actor.inner());
        Ok(())
    }
}