use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::AppError;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[serde(transparent)]
pub struct Phone(String);

impl Phone {
    pub fn new(phone: String) -> Self {
        Self(phone)
    }
}

impl<'de> Deserialize<'de> for Phone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // 1. Single pass: filter only digits into a new string
        let mut cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        // 2. Handle the +1 case efficiently
        // If the user provided "+1 (555) ...", 'cleaned' is now "1555..."
        if cleaned.len() == 11 && cleaned.starts_with('1') {
            cleaned.remove(0); // Removes the leading '1' in-place
        }
        Ok(Phone::new(cleaned))
    }
}

impl std::fmt::Display for Phone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "server")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type, TypeInfo};

    use super::Phone;

    impl Type<Postgres> for Phone {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty) || ty.name().eq_ignore_ascii_case("citext")
        }
    }

    impl<'q> Encode<'q, Postgres> for Phone {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for Phone {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let s = <String as Decode<Postgres>>::decode(value)?;
            Ok(Phone::new(s))
        }
    }
}

impl Validate for Phone {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let s = &self.0;

        // Basic format check
        if s.len() != 10 || !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::form_field_error(
                "phone",
                "Phone number must be 10 digits".into(),
            ));
        }

        // NANP specific check
        let bytes = s.as_bytes();
        if bytes[0] < b'2' || bytes[3] < b'2' {
            return Err(AppError::form_field_error(
                "phone",
                "Invalid North American Area or Exchange code".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use crate::types::Phone;

    #[test]
    fn should_cleanup() {
        let phone: Phone = serde_json::from_str("\"+1 (555) 123-4567\"").unwrap();
        assert_eq!(phone.to_string(), "5551234567");
    }

    #[test]
    fn should_fail_on_invalid_phone_length() {
        let result = serde_json::from_str::<Phone>("\"12345\"");
        assert!(result.is_ok(),);

        let phone = result.unwrap();
        let validation_result = phone.validate();
        assert!(validation_result.is_err(),);
        assert_eq!(
            validation_result.unwrap_err().to_string(),
            "phone: Phone number must be 10 digits".to_string()
        );
    }

    #[test]
    fn should_fail_on_invalid_phone() {
        let result = serde_json::from_str::<Phone>("\"1234567890\"");
        assert!(result.is_ok(),);

        let phone = result.unwrap();
        let validation_result = phone.validate();
        assert!(validation_result.is_err(),);
        assert_eq!(
            validation_result.unwrap_err().to_string(),
            "phone: Invalid North American Area or Exchange code".to_string()
        );
    }

    #[test]
    fn should_pass_valid_phone() {
        let result = serde_json::from_str::<Phone>("\"(234) 567-8901\"");
        assert!(result.is_ok(),);

        let phone = result.unwrap();
        let validation_result = phone.validate();
        assert!(validation_result.is_ok(),);
    }

    #[test]
    fn should_pass_valid_phone_without_space_and_brackets() {
        let result = serde_json::from_str::<Phone>("\"2345678901\"");
        assert!(result.is_ok(),);

        let phone = result.unwrap();
        let validation_result = phone.validate();
        assert!(validation_result.is_ok(),);
    }
}
