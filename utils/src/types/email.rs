use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateEmail, ValidationErrors};

use crate::AppError;
use crate::text_normalizer::remove_extra_space;

#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Self {
        Self(remove_extra_space(&email.to_lowercase()))
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "server")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type, TypeInfo};

    use super::Email;

    impl Type<Postgres> for Email {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty) || ty.name().eq_ignore_ascii_case("citext")
        }
    }

    impl<'q> Encode<'q, Postgres> for Email {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for Email {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <String as Decode<Postgres>>::decode(value)?;
            Ok(Email::new(s))
        }
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Email::new(s))
    }
}

impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if self.0.is_empty() {
            Err(AppError::form_field_error(
                "email",
                "Email is required".into(),
            ))
        } else if !self.0.validate_email() {
            Err(AppError::form_field_error(
                "email",
                "Enter a valid email address".into(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use crate::types::Email;

    #[test]
    fn should_fail_when_invalid_email_format() {
        let email = Email::new("invalid_email".to_string()).validate();
        assert!(email.is_err());
        assert_eq!(
            email.unwrap_err().to_string(),
            "email: Enter a valid email address".to_string()
        );
    }

    #[test]
    fn should_fail_when_empty() {
        let email = Email::new("".to_string()).validate();
        assert!(email.is_err());
        assert_eq!(
            email.unwrap_err().to_string(),
            "email: Email is required".to_string()
        );
    }

    #[test]
    fn should_pass_when_valid_email_is_used() {
        let email = Email::new("hola@gmail.com".to_string()).validate();
        assert!(email.is_ok());
    }

    #[test]
    fn should_remove_space_and_lower_case() {
        let email = Email::new("HoLa@gmail.com ".to_string());
        assert_eq!(email.to_string(), "hola@gmail.com");
    }
}
