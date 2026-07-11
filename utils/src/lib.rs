mod error;
pub mod format;

extern crate self as utils;

// Re-exports
pub use error::DsError;
pub type Result<T> = std::result::Result<T, DsError>;
