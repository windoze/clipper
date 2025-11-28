pub mod api;
pub mod config;
pub mod error;
pub mod state;
pub mod websocket;

// TLS and ACME modules (feature-gated)
#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "acme")]
pub mod acme;

pub mod cert_storage;

pub use config::{Cli, ServerConfig};
pub use error::{Result, ServerError};
pub use state::AppState;

#[cfg(feature = "tls")]
pub use tls::{TlsManager, TlsState};

#[cfg(feature = "acme")]
pub use acme::AcmeManager;
