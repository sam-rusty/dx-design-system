use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::AppError;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Password(String);

impl Password {
    pub fn new(password: String) -> Self {
        Self(password)
    }

    pub fn plain_text(&self) -> &str {
        &self.0
    }
}

impl Validate for Password {
    /// must be 8 characters long, only alpha-numeric characters
    fn validate(&self) -> Result<(), ValidationErrors> {
        let password = self.0.as_str();
        if password.is_empty() {
            return Err(AppError::form_field_error(
                "password",
                "Password is required".into(),
            ));
        }

        let mut length = 0;
        let mut has_uppercase = false;
        let mut has_lowercase = false;
        let mut has_digit = false;
        let mut has_special_char = false;

        for c in password.chars() {
            length += 1;
            if c.is_uppercase() {
                has_uppercase = true;
            } else if c.is_lowercase() {
                has_lowercase = true;
            }
            if c.is_ascii_digit() {
                has_digit = true;
            }
            if !c.is_alphanumeric() {
                has_special_char = true;
            }
        }

        if length >= 8 && has_uppercase && has_lowercase && has_digit && has_special_char {
            return Ok(());
        }

        Err(AppError::form_field_error(
            "password",
            "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, one number, and one special character".into(),
        ))
    }
}
