pub mod models;
pub mod indexer;
pub mod error;
pub mod storage;

pub use error::{IndexerError, Result};
pub use indexer::ClipperIndexer;
pub use models::{ClipboardEntry, SearchFilters};
