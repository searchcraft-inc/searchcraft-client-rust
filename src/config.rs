//! Client configuration and API-key selection.
//!
//! Use [`Config`] (via its builder or [`Config::new`]) to supply the
//! endpoint URL, API keys, timeout, and optional custom headers.

use std::collections::HashMap;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::error::Error;

/// Default request timeout (30 seconds).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The kind of operation, used to select the correct API key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Read operations (search, get).
    Read,
    /// Write / ingest operations (index documents, delete).
    Write,
    /// Administrative operations (manage auth keys, self-hosted).
    Admin,
}

/// Validated, immutable client configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the Searchcraft endpoint (trailing slash stripped).
    pub(crate) base_url: Url,
    /// API key used for read operations.
    pub(crate) read_key: Option<SecretString>,
    /// API key used for write / ingest operations.
    pub(crate) ingest_key: Option<SecretString>,
    /// API key used for admin operations (self-hosted only).
    pub(crate) admin_key: Option<SecretString>,
    /// Per-request timeout.
    pub(crate) timeout: Duration,
    /// Extra headers sent with every request.
    pub(crate) headers: HashMap<String, String>,
}

impl Config {
    /// Creates a new configuration, validating the endpoint URL.
    ///
    /// At least one of `read_key` or `ingest_key` must be provided. Admin
    /// endpoints additionally need [`with_admin_key`](Self::with_admin_key).
    ///
    /// ```
    /// # fn main() -> searchcraft::error::Result<()> {
    /// use std::time::Duration;
    /// use searchcraft::Config;
    ///
    /// let config = Config::new("https://my-instance.searchcraft.io", Some("sc-read"), None::<String>)?
    ///     .with_timeout(Duration::from_secs(10))
    ///     .with_header("X-Request-Source", "my-service");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] if `endpoint_url` cannot be parsed as
    /// a URL, or if neither key is supplied.
    pub fn new(
        endpoint_url: &str,
        read_key: Option<impl Into<String>>,
        ingest_key: Option<impl Into<String>>,
    ) -> crate::error::Result<Self> {
        let base_url = normalize_url(endpoint_url)?;

        let read_key = read_key.map(|k| SecretString::from(k.into()));
        let ingest_key = ingest_key.map(|k| SecretString::from(k.into()));

        if read_key.is_none() && ingest_key.is_none() {
            return Err(Error::Configuration(
                "at least one of read_key or ingest_key must be provided".into(),
            ));
        }

        Ok(Self {
            base_url,
            read_key,
            ingest_key,
            admin_key: None,
            timeout: DEFAULT_TIMEOUT,
            headers: HashMap::new(),
        })
    }

    /// Set the admin key (required for self-hosted auth management).
    #[must_use]
    pub fn with_admin_key(mut self, key: impl Into<String>) -> Self {
        self.admin_key = Some(SecretString::from(key.into()));
        self
    }

    /// Override the default request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a custom header sent with every request.
    #[must_use]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Returns the normalized base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Returns the per-request timeout.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Select the API key for the given [`Operation`].
    ///
    /// Returns an error if the required key is not configured.
    pub(crate) fn api_key_for(&self, op: Operation) -> crate::error::Result<&str> {
        let (key, label) = match op {
            Operation::Read => (&self.read_key, "read_key"),
            Operation::Write => (&self.ingest_key, "ingest_key"),
            Operation::Admin => (&self.admin_key, "admin_key"),
        };
        key.as_ref()
            .map(ExposeSecret::expose_secret)
            .ok_or_else(|| Error::Configuration(format!("{label} is required for this operation")))
    }
}

/// Parse and normalize the endpoint URL (strip trailing slash).
fn normalize_url(raw: &str) -> crate::error::Result<Url> {
    let trimmed = raw.trim_end_matches('/');
    // Append a trailing slash so `Url::join` works correctly with relative paths.
    let with_slash = format!("{trimmed}/");
    Url::parse(&with_slash)
        .map_err(|e| Error::Configuration(format!("invalid endpoint URL '{raw}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config_strips_trailing_slash() {
        let cfg = Config::new("https://api.example.com/", Some("rk"), None::<String>).unwrap();
        // base_url always ends with `/` for Url::join, but the path should be just `/`
        assert_eq!(cfg.base_url.as_str(), "https://api.example.com/");
    }

    #[test]
    fn valid_config_without_trailing_slash() {
        let cfg = Config::new("https://api.example.com", Some("rk"), None::<String>).unwrap();
        assert_eq!(cfg.base_url.as_str(), "https://api.example.com/");
    }

    #[test]
    fn missing_keys_is_error() {
        let err = Config::new("https://api.example.com", None::<String>, None::<String>);
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("at least one"), "got: {msg}");
    }

    #[test]
    fn invalid_url_is_error() {
        let err = Config::new("not a url", Some("rk"), None::<String>);
        assert!(err.is_err());
    }

    #[test]
    fn api_key_selection() {
        let cfg = Config::new("https://x.com", Some("read"), Some("write"))
            .unwrap()
            .with_admin_key("admin");

        assert_eq!(cfg.api_key_for(Operation::Read).unwrap(), "read");
        assert_eq!(cfg.api_key_for(Operation::Write).unwrap(), "write");
        assert_eq!(cfg.api_key_for(Operation::Admin).unwrap(), "admin");
    }

    #[test]
    fn missing_admin_key_is_error() {
        let cfg = Config::new("https://x.com", Some("rk"), None::<String>).unwrap();
        let err = cfg.api_key_for(Operation::Admin);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("admin_key"));
    }

    #[test]
    fn timeout_default_and_override() {
        let cfg = Config::new("https://x.com", Some("rk"), None::<String>).unwrap();
        assert_eq!(cfg.timeout(), Duration::from_secs(30));

        let cfg = cfg.with_timeout(Duration::from_secs(5));
        assert_eq!(cfg.timeout(), Duration::from_secs(5));
    }

    #[test]
    fn custom_headers() {
        let cfg = Config::new("https://x.com", Some("rk"), None::<String>)
            .unwrap()
            .with_header("X-Custom", "value");
        assert_eq!(cfg.headers.get("X-Custom").unwrap(), "value");
    }
}
