pub mod enums;
mod error;
pub mod format;
#[cfg(feature = "web")]
mod local_storage;
pub mod text_normalizer;

extern crate self as utils;

pub mod filter;
pub mod types;

#[cfg(feature = "server")]
pub mod test;

#[cfg(feature = "server")]
mod tracing;

#[cfg(feature = "server")]
mod db;

#[cfg(feature = "server")]
pub(crate) mod db_error_messages;

// Re-exports
pub use enums::ActivityKind;
pub use error::AppError;
pub use filter::{ColumnType, EnumWidget, FilterClause, FilterColumns, FilterOp, FilterSet, FilterType};
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(feature = "server")]
pub use db::{Connection, Row};
#[cfg(feature = "web")]
pub use local_storage::LocalStorage;
#[cfg(feature = "server")]
pub(crate) use tracing::error;
