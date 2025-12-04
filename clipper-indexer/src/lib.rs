pub mod error;
pub mod indexer;
pub mod models;
pub mod storage;

pub use error::{IndexerError, Result};
pub use indexer::ClipperIndexer;
pub use models::{ClipboardEntry, PagedResult, PagingParams, SearchFilters, ShortUrl};
