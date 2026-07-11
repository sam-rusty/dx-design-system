#![allow(unused)]

use std::fmt::Display;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::Result;

macro_rules! define_id_type {
    ($name:ident, $error_msg:expr) => {
        #[derive(
            Serialize,
            Deserialize,
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            Default,
            PartialOrd,
            Ord,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new(value: Uuid) -> Self {
                Self(value)
            }

            pub fn value(self) -> Uuid {
                self.0
            }

            pub fn is_nil(self) -> bool {
                self.0 == Uuid::nil()
            }

            pub fn generate() -> Self {
                #[cfg(feature = "server")]
                {
                    Self::new(Uuid::now_v7())
                }
                #[cfg(not(feature = "server"))]
                {
                    Self::new(Uuid::new_v4())
                }
            }
        }

        #[cfg(feature = "server")]
        impl $name {
            pub fn from_row(row: &crate::db::Row, idx: usize) -> crate::Result<Self> {
                use sqlx::Row;
                let id: sqlx::types::Uuid = row.try_get_unchecked(idx).map_err(|e| {
                    crate::AppError::InternalServer(format!(
                        concat!(stringify!($name), " decode[{}]: {}"),
                        idx, e
                    ))
                })?;
                Ok(Self(id))
            }

            pub fn from_row_opt(row: &crate::db::Row, idx: usize) -> crate::Result<Option<Self>> {
                use sqlx::Row;
                let opt: Option<sqlx::types::Uuid> = row.try_get_unchecked(idx).map_err(|e| {
                    crate::AppError::InternalServer(format!(
                        concat!(stringify!($name), " decode_opt[{}]: {}"),
                        idx, e
                    ))
                })?;
                Ok(opt.map(Self))
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = crate::AppError;

            fn try_from(value: &str) -> Result<Self> {
                Ok(Self(Uuid::parse_str(value)?))
            }
        }

        impl std::str::FromStr for $name {
            type Err = crate::AppError;

            fn from_str(s: &str) -> Result<Self> {
                Self::try_from(s)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        #[cfg(feature = "server")]
        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <sqlx::types::Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
            }
        }

        #[cfg(feature = "server")]
        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <sqlx::types::Uuid as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        #[cfg(feature = "server")]
        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
                Ok(Self(
                    <sqlx::types::Uuid as sqlx::Decode<sqlx::Postgres>>::decode(value)?,
                ))
            }
        }

        #[cfg(feature = "server")]
        impl sqlx::postgres::PgHasArrayType for $name {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                <sqlx::types::Uuid as sqlx::postgres::PgHasArrayType>::array_type_info()
            }
        }
    };
}

define_id_type!(PersonId, "Invalid Contact ID");
define_id_type!(CardId, "Invalid card ID");
define_id_type!(CollectionId, "Invalid group ID");

// ── Scheduling ───────────────────────────────────────────────────────────────

define_id_type!(ScheduleId, "Invalid schedule ID");
define_id_type!(BookingTypeId, "Invalid booking type ID");
