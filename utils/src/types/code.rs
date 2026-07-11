use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::AppError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Code(String);

impl Code {
    pub fn new(code: String) -> Self {
        Self(code)
    }

    pub fn value(self) -> String {
        self.0
    }
}

impl From<&str> for Code {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for Code {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "server")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type, TypeInfo};

    use super::Code;

    impl Type<Postgres> for Code {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty) || ty.name().eq_ignore_ascii_case("citext")
        }
    }

    impl<'q> Encode<'q, Postgres> for Code {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for Code {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <String as Decode<Postgres>>::decode(value)?;
            Ok(Code::new(s))
        }
    }
}

impl Validate for Code {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if self.0.len() != 8 || !self.0.chars().all(|c| c.is_ascii_alphanumeric()) {
            Err(AppError::form_field_error(
                "code",
                "Code must be 8 characters long and contain only alphanumeric characters".into(),
            ))
        } else {
            Ok(())
        }
    }
}
