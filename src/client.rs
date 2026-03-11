//! The top-level [`SearchcraftClient`].
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
//! let request = QueryBuilder::fuzzy().term("laptop").limit(10).build_request();
//! let response = client.search_index::<serde_json::Value>("products", &request).await?;
//! # Ok(())
//! # }
//! ```

use crate::config::Config;
use crate::error;
use crate::transport::HttpTransport;

/// Async client for the Searchcraft API.
///
/// Construct via [`SearchcraftClient::new`] or
/// [`SearchcraftClient::from_config`].
///
/// Search methods are provided via the [`search`](crate::search) module.
#[derive(Debug, Clone)]
pub struct SearchcraftClient {
    pub(crate) transport: HttpTransport,
}

impl SearchcraftClient {
    /// Create a client from raw parameters.
    ///
    /// This is a convenience wrapper around [`Config::new`] +
    /// [`SearchcraftClient::from_config`].
    pub fn new(
        endpoint_url: &str,
        read_key: Option<impl Into<String>>,
        ingest_key: Option<impl Into<String>>,
    ) -> error::Result<Self> {
        let config = Config::new(endpoint_url, read_key, ingest_key)?;
        Self::from_config(config)
    }

    /// Create a client from a pre-built [`Config`].
    pub fn from_config(config: Config) -> error::Result<Self> {
        let transport = HttpTransport::new(config)?;
        Ok(Self { transport })
    }

    /// Returns a reference to the underlying transport (for sub-API modules).
    #[allow(dead_code)]
    pub(crate) fn transport(&self) -> &HttpTransport {
        &self.transport
    }

    /// Returns a reference to the client configuration.
    pub fn config(&self) -> &Config {
        self.transport.config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation_succeeds() {
        let client = SearchcraftClient::new("https://api.example.com", Some("rk"), None::<String>);
        assert!(client.is_ok());
    }

    #[test]
    fn client_creation_fails_without_keys() {
        let client =
            SearchcraftClient::new("https://api.example.com", None::<String>, None::<String>);
        assert!(client.is_err());
    }

    #[test]
    fn client_from_config() {
        let config = Config::new("https://api.example.com", Some("rk"), None::<String>).unwrap();
        let client = SearchcraftClient::from_config(config);
        assert!(client.is_ok());
    }
}
