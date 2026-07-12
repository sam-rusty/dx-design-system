mod error;
pub mod format;

// Re-exports
pub use error::DsError;
pub type Result<T> = std::result::Result<T, DsError>;
