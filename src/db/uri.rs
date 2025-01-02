use std::{fmt::{Debug, Display}, ops::Deref};

use activitypub_federation::{fetch::object_id::ObjectId, traits::Object};
use serde::Deserialize;
use sqlx::{Decode, Encode, Type};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uri(Url);

impl Uri {
    pub fn as_url(&self) -> &Url {
        &self.0
    }
}

impl Deref for Uri {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Type<sqlx::Sqlite> for Uri {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        <String as Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'a> Decode<'a, sqlx::Sqlite> for Uri {
    fn decode(value: sqlx::sqlite::SqliteValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(Self(Url::parse(s)?))
    }
}

impl<'a> Encode<'a, sqlx::Sqlite> for Uri {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'a>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.0.to_string();
        <String as Encode<sqlx::Sqlite>>::encode(s, buf)
    }
}

impl TryFrom<String> for Uri {
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Url::parse(&value)?))
    }
}

impl From<Url> for Uri {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

impl Into<Url> for Uri {
    fn into(self) -> Url {
        self.0
    }
}


impl<'de, T> Into<ObjectId<T>> for Uri
where
    T: Object + Send + 'static,
    for<'de2> <T as Object>::Kind: Deserialize<'de2>
{
    fn into(self) -> ObjectId<T> {
        self.0.into()
    }
}
