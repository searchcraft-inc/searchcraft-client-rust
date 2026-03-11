//! # Searchcraft
//!
//! An async Rust client for the [Searchcraft](https://searchcraft.io) search API.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn example() -> searchcraft::error::Result<()> {
//! use searchcraft::SearchcraftClient;
//!
//! let client = SearchcraftClient::new(
//!     "https://my-instance.searchcraft.io",
//!     Some("sc-read-key"),
//!     None::<String>,
//! )?;
//!
//! // Use client.search(), client.indices(), etc. (coming soon)
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature flags
//!
//! | Feature      | Default | Description                        |
//! |--------------|---------|------------------------------------|
//! | `rustls`     | ✅      | Use rustls for TLS                 |
//! | `native-tls` | ❌      | Use the platform's native TLS      |

pub mod client;
pub mod config;
pub mod error;
pub mod transport;
pub mod types;

// Re-exports for ergonomic top-level access.
pub use client::SearchcraftClient;
pub use config::{Config, Operation};
pub use error::Error;
