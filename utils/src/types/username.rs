use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::AppError;
use crate::text_normalizer::remove_extra_space;

fn is_valid_username(username: &str) -> bool {
    let len = username.len();
    if !(4..=20).contains(&len) {
        return false;
    }
    username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
    pub fn new(username: String) -> Self {
        Self(remove_extra_space(&username.to_lowercase()))
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Username::new(s))
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "server")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type, TypeInfo};

    use super::Username;

    impl Type<Postgres> for Username {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty) || ty.name().eq_ignore_ascii_case("citext")
        }
    }

    impl<'q> Encode<'q, Postgres> for Username {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for Username {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <String as Decode<Postgres>>::decode(value)?;
            Ok(Username::new(s))
        }
    }
}

impl Validate for Username {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if self.0.is_empty() || !is_valid_username(&self.0) {
            Err(AppError::form_field_error("username", "Username must be between 4 to 20 characters long and can only contain letters, numbers, underscores and hyphens".into()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    #[test]
    fn should_fail_to_validate_invalid_usernames() {
        let invalid_usernames = vec![
            "ab",                                    // Too short
            "thisisaverylongusernameexceedinglimit", // Too long
            "invalid username",                      // Contains space
            "invalid@username!",                     // Contains special characters
            "user$name",                             // Contains special character
            " ",                                     // space only
        ];

        for username in invalid_usernames {
            let value = super::Username::new(username.to_string());
            assert!(value.validate().is_err());
        }
    }

    #[test]
    fn should_pass_to_validate_valid_usernames() {
        let valid_usernames = vec![
            "user_123",
            "username-456",
            "UserName",
            "userName_789",
            "USER-name",
            "abcd",                 // Minimum length
            "abcdefghijklmnopqrst", // Maximum length
            "  hola  ",             // Leading and trailing spaces should be removed
        ];

        for username in valid_usernames {
            let value = super::Username::new(username.to_string());
            assert!(value.validate().is_ok());
        }
    }
}
