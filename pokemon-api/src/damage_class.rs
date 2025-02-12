use async_graphql::Enum;
use sqlx::error::BoxDynError;
use sqlx::{Database, Decode, Sqlite, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum DamageClass {
    Physical,
    Special,
    Status,
}

impl TryFrom<i64> for DamageClass {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DamageClass::Status),
            2 => Ok(DamageClass::Physical),
            3 => Ok(DamageClass::Special),
            _ => Err("Unknown damage class.")
        }
    }
}

impl Type<Sqlite> for DamageClass {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'a> Decode<'a, Sqlite> for DamageClass {
    fn decode(value: <Sqlite as Database>::ValueRef<'a>) -> Result<Self, BoxDynError> {
        let value = <i64 as Decode<Sqlite>>::decode(value)?;
        Ok(DamageClass::try_from(value)?)
    }
}