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

impl DsError {
    pub fn form_field_error(field: &'static str, message: String) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let err = validator::ValidationError::new("").with_message(Cow::Owned(message));
        errors.add(field, err);
        errors
    }
}
