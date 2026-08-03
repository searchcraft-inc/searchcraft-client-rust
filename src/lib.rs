//! # Searchcraft
//!
//! An async Rust client for the [Searchcraft](https://searchcraft.io) search API.
//!
//! This crate provides a typed, ergonomic interface to the Searchcraft search
//! and management APIs. It is built on [`reqwest`] and supports both
//! `rustls` (default) and `native-tls` backends.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn example() -> searchcraft::error::Result<()> {
//! use searchcraft::SearchcraftClient;
//! use searchcraft::search::query::QueryBuilder;
//!
//! let client = SearchcraftClient::new(
//!     "https://my-instance.searchcraft.io",
//!     Some("sc-read-key"),
//!     None::<String>,
//! )?;
//!
//! let request = QueryBuilder::fuzzy()
//!     .term("laptop")
//!     .limit(10)
//!     .build_request();
//!
//! let response = client
//!     .search_index::<serde_json::Value>("products", &request)
//!     .await?;
//!
//! println!("Found {} results", response.data.count);
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`search`] — Search queries and the [`QueryBuilder`](search::query::QueryBuilder).
//! - [`admin`] — Index, document, federation, auth-key, and other management APIs.
//! - [`config`] — Client configuration ([`Config`]) and API-key selection.
//! - [`error`] — Error types and the crate-wide [`Result`](error::Result) alias.
//! - [`types`] — Shared types such as [`SortDirection`](types::SortDirection).
//! - [`transport`] — Low-level HTTP transport (not typically used directly).
//!
//! ## Building queries
//!
//! The [`QueryBuilder`](search::query::QueryBuilder) provides a chainable API
//! for constructing search requests. Each method returns a new builder value,
//! leaving the original unchanged:
//!
//! ```
//! use searchcraft::search::query::QueryBuilder;
//! use searchcraft::types::SortDirection;
//!
//! let request = QueryBuilder::fuzzy()
//!     .term("laptop")
//!     .and("gaming")
//!     .not("refurbished")
//!     .order_by("price", SortDirection::Asc)
//!     .limit(20)
//!     .build_request();
//! ```
//!
//! ## Streaming AI summaries
//!
//! On engine 0.10.0+, [`search_summary`](SearchcraftClient::search_summary)
//! streams an LLM-generated summary of a query's results as Server-Sent Events.
//! Check [`get_index_capabilities`](SearchcraftClient::get_index_capabilities)
//! first — the endpoint needs AI features enabled on the index:
//!
//! ```no_run
//! # async fn example() -> searchcraft::error::Result<()> {
//! # let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
//! # let req = searchcraft::search::query::QueryBuilder::fuzzy().term("laptop").build_request();
//! use searchcraft::search::types::SummaryStreamEvent;
//! use searchcraft::search::StreamExt;
//!
//! let mut stream = client.search_summary("products", &req).await?;
//! while let Some(event) = stream.next().await {
//!     if let SummaryStreamEvent::Delta(d) = event {
//!         print!("{}", d.content);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Admin operations
//!
//! Management methods are available directly on [`SearchcraftClient`] when the
//! appropriate key is configured:
//!
//! ```no_run
//! # async fn example() -> searchcraft::error::Result<()> {
//! use searchcraft::SearchcraftClient;
//! use searchcraft::admin::types::IndexConfig;
//!
//! let client = SearchcraftClient::new(
//!     "https://my-instance.searchcraft.io",
//!     Some("sc-read-key"),
//!     Some("sc-ingest-key"),
//! )?;
//!
//! // Create an index
//! client.create_index("products", &IndexConfig::default()).await?;
//!
//! // Insert a document
//! let doc = serde_json::json!({"id": "1", "title": "Laptop", "price": 999});
//! client.insert_document("products", &doc).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error handling
//!
//! All fallible operations return [`error::Result<T>`](error::Result). Use
//! pattern matching or [`Error::is_retryable`] to decide how to handle
//! failures:
//!
//! ```no_run
//! # async fn example() -> searchcraft::error::Result<()> {
//! # let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
//! # let req = searchcraft::search::query::QueryBuilder::fuzzy().term("x").build_request();
//! match client.search_index::<serde_json::Value>("products", &req).await {
//!     Ok(resp) => println!("{} hits", resp.data.count),
//!     Err(e) if e.is_retryable() => eprintln!("transient: {e}"),
//!     Err(e) => return Err(e),
//! }
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

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

pub mod admin;
pub mod client;
pub mod config;
pub mod error;
pub mod search;
pub mod transport;
pub mod types;

// Re-exports for ergonomic top-level access.
pub use client::SearchcraftClient;
pub use config::{Config, Operation};
pub use error::Error;

/// Compiles the README's Rust examples as doctests so they cannot drift out of
/// sync with the API. Not part of the public surface.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;
