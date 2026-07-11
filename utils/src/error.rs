use std::borrow::Cow;

use dioxus::prelude::ServerFnError;
use macros::on_server;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidationErrors;

on_server! {
    use dioxus::fullstack::{AsStatusCode, FullstackContext, MakeAxumError, ServerFnDecoder};
    use http::StatusCode;
}

#[derive(Error, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "msg")]
pub enum AppError {
    #[error("Missing or expired Session Cookie")]
    MissingOrExpireCookie,
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    ServiceUnavailable(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    InternalServer(String),
    #[error("{0}")]
    Validation(String, ValidationErrors),
}

impl AppError {
    /// Recover a typed `AppError` from the dioxus client's fallback error message. When the client
    /// can't decode a non-2xx body into the `ErrorPayload` envelope, it wraps the raw body as
    /// `HTTP <code>: <body>`. Our error bodies are plain `AppError` JSON (`{"kind":..,"msg":..}`),
    /// so parse the embedded object back into the concrete variant; fall back to a generic internal
    /// error when no `AppError` JSON is present.
    fn from_wire(message: &str) -> AppError {
        if let Some(start) = message.find('{')
            && let Ok(err) = serde_json::from_str::<AppError>(&message[start..])
        {
            return err;
        }
        #[cfg(feature = "server")]
        crate::error!("{message}");
        AppError::InternalServer("Internal Server Error".to_string())
    }

    /// to show error in a specific form field
    pub fn form_field_error(field: &'static str, message: String) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let err = validator::ValidationError::new("").with_message(Cow::Owned(message));
        errors.add(field, err);
        errors
    }

    on_server! {
        pub fn status_code(&self) -> http::StatusCode {
            use http::StatusCode;

            match self {
                AppError::MissingOrExpireCookie | AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                AppError::Forbidden(_) => StatusCode::FORBIDDEN,
                AppError::NotFound(_) => StatusCode::NOT_FOUND,
                AppError::BadRequest(_) | AppError::Validation(_, _) => StatusCode::BAD_REQUEST,
                AppError::Conflict(_) => StatusCode::CONFLICT,
                AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
                AppError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }

        pub fn throw(self) -> AppError {
            self.apply_status();
            self
        }

        fn apply_status(&self) {
            if let Some(mut ctx) = FullstackContext::current() {
                use dioxus::prelude::HttpError;

                ctx.set_current_http_status(HttpError {
                    status: self.status_code(),
                    message: Some(self.to_string()),
                });
            }
        }
    }

    #[cfg(not(feature = "server"))]
    fn apply_status(&self) {}
}

on_server! {
    impl axum::response::IntoResponse for AppError {
        fn into_response(self) -> axum::response::Response {
            use axum::http::header;

            let status = self.status_code();

            // Sanitize internal errors — never leak the actual message
            let sanitized: AppError = match self {
                AppError::InternalServer(ref msg) => {
                    crate::error!("{msg}");
                    AppError::InternalServer("Internal Server Error".to_string())
                }
                other => other,
            };

            let body = serde_json::to_string(&sanitized).unwrap_or_else(|_| {
                r#"{"kind":"InternalServer","msg":"Internal Server Error"}"#.to_string()
            });

            (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
        }
    }

    /// Higher-priority `MakeAxumError` impl (4 `&`) that bypasses the default `ErrorPayload` wrapper
    /// and instead uses `AppError::IntoResponse` directly, so the wire format is plain
    /// `{"kind":"...", "msg":"..."}` JSON. The typed error is recovered on the client by
    /// [`AppError::from_wire`] via the `From<ServerFnError>` impl.
    impl<T> MakeAxumError<AppError> for &&&&ServerFnDecoder<Result<T, AppError>> {
        fn make_axum_error(
            self,
            result: Result<axum::response::Response, AppError>,
        ) -> axum::response::Response {
            use axum::response::IntoResponse as _;
            match result {
                Ok(resp) => resp,
                Err(err) => err.into_response(),
            }
        }
    }

    impl AsStatusCode for AppError {
        fn as_status_code(&self) -> StatusCode {
            use StatusCode;
            match self {
                AppError::MissingOrExpireCookie | AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                AppError::Forbidden(_) => StatusCode::FORBIDDEN,
                AppError::NotFound(_) => StatusCode::NOT_FOUND,
                AppError::BadRequest(_) | AppError::Validation(_, _) => StatusCode::BAD_REQUEST,
                AppError::Conflict(_) => StatusCode::CONFLICT,
                AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
                AppError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }

    impl From<std::env::VarError> for AppError {
        fn from(e: std::env::VarError) -> Self {
            crate::error!("error getting env variable: {e}");
            AppError::InternalServer("Internal Server Error".to_string())
        }
    }

    impl From<serde::de::value::Error> for AppError {
        fn from(value: serde::de::value::Error) -> Self {
            crate::error!("serde error: {value}");
            AppError::InternalServer("Internal Server Error".to_string())
        }
    }

    impl From<sqlx::Error> for AppError {
        fn from(value: sqlx::Error) -> Self {
            let err = crate::db_error_messages::ErrorFormatting::sqlx(value);
            err.apply_status();
            err
        }
    }
}

impl From<ServerFnError> for AppError {
    fn from(value: ServerFnError) -> Self {
        use ServerFnError;
        let err = match value {
            ServerFnError::ServerError { message, .. } => Self::from_wire(&message),
            ServerFnError::Args(e) => AppError::BadRequest(e),
            ServerFnError::MissingArg(e) => AppError::BadRequest(e),
            ServerFnError::Deserialization(e) => AppError::BadRequest(e),
            ServerFnError::Registration(e) => AppError::InternalServer(e),
            ServerFnError::Request(e) => AppError::InternalServer(e.to_string()),
            ServerFnError::Serialization(e) => AppError::InternalServer(e),
            ServerFnError::Response(e) => AppError::InternalServer(e),
            ServerFnError::StreamError(e) => AppError::InternalServer(e),
            ServerFnError::UnsupportedRequestMethod(e) => AppError::BadRequest(e),
            ServerFnError::MiddlewareError(e) => AppError::InternalServer(e),
        };
        err.apply_status();
        err
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        let err = Self::BadRequest(value.to_string());
        err.apply_status();
        err
    }
}

impl From<uuid::Error> for AppError {
    fn from(_value: uuid::Error) -> Self {
        #[cfg(feature = "server")]
        crate::error!("{_value}");
        AppError::InternalServer("Error Parsing Uuid".to_string())
    }
}

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        let err = Self::Validation("Validation failed".to_string(), value);
        err.apply_status();
        err
    }
}

impl From<strum::ParseError> for AppError {
    fn from(value: strum::ParseError) -> Self {
        let err = Self::BadRequest(value.to_string());
        err.apply_status();
        err
    }
}
