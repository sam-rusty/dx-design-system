mod error;
pub mod format;

extern crate self as utils;

// Re-exports
pub use error::AppError;
pub type Result<T> = std::result::Result<T, AppError>;
