//! HTTP transport layer for the Searchcraft API.
//!
//! [`HttpTransport`] wraps a [`reqwest::Client`] and handles
//! authentication headers, timeouts, and error mapping.

use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::{Config, Operation};
use crate::error::{self, Error};

/// Shared async HTTP transport.
///
/// Constructed once per [`crate::SearchcraftClient`] and reused for all
/// requests. Holds a connection-pooled [`reqwest::Client`].
#[derive(Debug, Clone)]
pub struct HttpTransport {
    client: Client,
    config: Config,
}

impl HttpTransport {
    /// Build a new transport from the given [`Config`].
    pub fn new(config: Config) -> error::Result<Self> {
        let mut builder = Client::builder().timeout(config.timeout());

        // Apply default headers from config.
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        for (name, value) in &config.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| Error::Configuration(format!("invalid header name '{name}': {e}")))?;
            let header_value = value.parse().map_err(|e| {
                Error::Configuration(format!("invalid header value for '{name}': {e}"))
            })?;
            default_headers.insert(header_name, header_value);
        }
        builder = builder.default_headers(default_headers);

        let client = builder
            .build()
            .map_err(|e| Error::Configuration(format!("failed to build HTTP client: {e}")))?;

        Ok(Self { client, config })
    }

    /// Send a request that expects a JSON response body.
    pub async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        operation: Operation,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> error::Result<T> {
        let url = self
            .config
            .base_url
            .join(path)
            .map_err(|e| Error::Configuration(format!("invalid path '{path}': {e}")))?;

        let api_key = self.config.api_key_for(operation)?;

        let mut req: RequestBuilder = self.client.request(method, url.as_str());
        req = req.header("Authorization", api_key);

        if let Some(b) = body {
            req = req.json(b);
        }

        let response = req.send().await.map_err(|e| {
            if e.is_timeout() {
                Error::Network("request timed out".into())
            } else if e.is_connect() {
                Error::Network(format!("connection failed: {e}"))
            } else {
                Error::from(e)
            }
        })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(error::map_status_error(status, body_text));
        }

        response.json::<T>().await.map_err(Error::from)
    }

    /// Send a request that returns no meaningful body (e.g. DELETE 204).
    pub async fn request_no_content(
        &self,
        method: Method,
        path: &str,
        operation: Operation,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> error::Result<()> {
        let url = self
            .config
            .base_url
            .join(path)
            .map_err(|e| Error::Configuration(format!("invalid path '{path}': {e}")))?;

        let api_key = self.config.api_key_for(operation)?;

        let mut req = self.client.request(method, url.as_str());
        req = req.header("Authorization", api_key);

        if let Some(b) = body {
            req = req.json(b);
        }

        let response = req.send().await.map_err(|e| {
            if e.is_timeout() {
                Error::Network("request timed out".into())
            } else if e.is_connect() {
                Error::Network(format!("connection failed: {e}"))
            } else {
                Error::from(e)
            }
        })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(error::map_status_error(status, body_text));
        }

        Ok(())
    }

    /// Send a request that expects a plain-text response body.
    pub async fn request_text(
        &self,
        method: Method,
        path: &str,
        operation: Operation,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> error::Result<String> {
        let url = self
            .config
            .base_url
            .join(path)
            .map_err(|e| Error::Configuration(format!("invalid path '{path}': {e}")))?;

        let api_key = self.config.api_key_for(operation)?;

        let mut req = self.client.request(method, url.as_str());
        req = req.header("Authorization", api_key);

        if let Some(b) = body {
            req = req.json(b);
        }

        let response = req.send().await.map_err(|e| {
            if e.is_timeout() {
                Error::Network("request timed out".into())
            } else if e.is_connect() {
                Error::Network(format!("connection failed: {e}"))
            } else {
                Error::from(e)
            }
        })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(error::map_status_error(status, body_text));
        }

        response.text().await.map_err(Error::from)
    }

    /// Returns a reference to the underlying configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }
}
