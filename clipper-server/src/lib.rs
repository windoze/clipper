pub mod api;
pub mod error;
pub mod state;
pub mod websocket;

pub use error::{Result, ServerError};
pub use state::AppState;
