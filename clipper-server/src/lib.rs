pub mod api;
pub mod auth;
pub mod cleanup;
pub mod config;
pub mod error;
pub mod parent_monitor;
pub mod state;
pub mod websocket;

// TLS and ACME modules (feature-gated)
#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "acme")]
pub mod acme;

pub mod cert_storage;

pub use auth::auth_middleware;
pub use cleanup::{run_clip_cleanup_task, run_short_url_cleanup_task};
pub use config::{AuthConfig, CleanupConfig, Cli, ServerConfig};
pub use error::{Result, ServerError};
pub use state::{AppState, ClipUpdate};

#[cfg(feature = "tls")]
pub use tls::{TlsManager, TlsState};

#[cfg(feature = "acme")]
pub use acme::AcmeManager;
