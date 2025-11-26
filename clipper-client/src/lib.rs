pub mod client;
pub mod error;
pub mod models;

pub use client::ClipperClient;
pub use error::{ClientError, Result};
pub use models::{Clip, ClipNotification, CreateClipRequest, SearchFilters, UpdateClipRequest};
