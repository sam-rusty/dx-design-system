use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Clone, Debug, Serialize, Deserialize)]
pub enum DsError {
    #[error("{0}")]
    InternalServer(String),
    #[error("{0}")]
    Other(String),
    #[error("{0}")]
    Validation(String, ValidationErrors),
}

impl From<String> for DsError {
    fn from(message: String) -> Self {
        DsError::Other(message)
    }
}

impl From<&str> for DsError {
    fn from(message: &str) -> Self {
        DsError::Other(message.to_string())
    }
}

impl From<serde_json::Error> for DsError {
    fn from(err: serde_json::Error) -> Self {
        DsError::Other(err.to_string())
    }
}

/// Field errors with no headline message; pair with [`DsError::Validation`] directly when
/// you have one.
impl From<ValidationErrors> for DsError {
    fn from(errors: ValidationErrors) -> Self {
        DsError::Validation(String::new(), errors)
    }
}

impl DsError {
    /// Builds a single-field [`ValidationErrors`] (not a `DsError`): wrap it via
    /// `DsError::Validation(msg, errors)` or `.into()`. The name is kept for compatibility.
    pub fn form_field_error(field: &'static str, message: String) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let err = validator::ValidationError::new("").with_message(Cow::Owned(message));
        errors.add(field, err);
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_strings_and_serde_errors_land_in_other() {
        assert!(matches!(DsError::from("boom"), DsError::Other(m) if m == "boom"));
        assert!(matches!(DsError::from(String::from("x")), DsError::Other(m) if m == "x"));
        let err = serde_json::from_str::<u8>("nope").unwrap_err();
        let converted: DsError = err.into();
        assert!(matches!(converted, DsError::Other(m) if !m.is_empty()));
    }

    #[test]
    fn from_validation_errors_keeps_field_errors() {
        let errors = DsError::form_field_error("email", "Invalid".into());
        let converted: DsError = errors.into();
        match converted {
            DsError::Validation(msg, errs) => {
                assert!(msg.is_empty());
                assert!(errs.field_errors().contains_key("email"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn question_mark_converts_into_result() {
        fn parse() -> crate::Result<u8> {
            Ok(serde_json::from_str::<u8>("7")?)
        }
        assert_eq!(parse().unwrap(), 7);
    }
}
