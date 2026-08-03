//! HTTP transport layer for the Searchcraft API.
//!
//! [`HttpTransport`] wraps a [`reqwest::Client`] and handles authentication
//! headers, timeouts, and error mapping. It is exposed for callers that need
//! to reach an endpoint this client does not wrap yet; ordinary use goes
//! through the methods on [`SearchcraftClient`](crate::SearchcraftClient).

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
    /// Client used for streaming responses. Bounds only the time to establish
    /// a connection, so a long-lived stream is never cut off mid-flight.
    stream_client: Client,
    config: Config,
}

impl HttpTransport {
    /// Builds a new transport from the given [`Config`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] if a custom header name or value is
    /// invalid, or if the underlying HTTP client cannot be built.
    pub fn new(config: Config) -> error::Result<Self> {
        // Apply default headers from config.
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        for (name, value) in &config.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| Error::Configuration(format!("invalid header name '{name}': {e}")))?;
            let header_value = value.parse().map_err(|e| {
                Error::Configuration(format!("invalid header value for '{name}': {e}"))
            })?;
            default_headers.insert(header_name, header_value);
        }

        let client = Client::builder()
            .timeout(config.timeout())
            .default_headers(default_headers.clone())
            .build()
            .map_err(|e| Error::Configuration(format!("failed to build HTTP client: {e}")))?;

        // The configured timeout bounds the whole request/response cycle, which
        // would abort a summary stream partway through. For streaming we bound
        // connection setup instead and let the body run as long as the server
        // keeps sending.
        let stream_client = Client::builder()
            .connect_timeout(config.timeout())
            .default_headers(default_headers)
            .build()
            .map_err(|e| Error::Configuration(format!("failed to build HTTP client: {e}")))?;

        Ok(Self {
            client,
            stream_client,
            config,
        })
    }

    /// Sends a request and returns the `data` payload of the Searchcraft
    /// response envelope, deserialized into `T`.
    ///
    /// Every Searchcraft endpoint wraps its payload as
    /// `{ "status": 200, "data": ... }`, so this is the method endpoint
    /// wrappers should use. Reach for [`request`](Self::request) only when you
    /// need the envelope itself.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] if the key required by `operation` is
    /// not set, [`Error::Network`] on a timeout or connection failure, the
    /// mapped status error for a non-2xx response, or [`Error::Http`] if the
    /// payload does not deserialize into `T`.
    pub async fn request_data<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        operation: Operation,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> error::Result<T> {
        let envelope: crate::admin::types::ApiResponse<T> =
            self.request(method, path, operation, body).await?;
        Ok(envelope.data)
    }

    /// Sends a request and deserializes the whole JSON response body into `T`.
    ///
    /// Callers usually want [`request_data`](Self::request_data), which unwraps
    /// the `{ status, data }` envelope every endpoint returns.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] if the key required by `operation` is
    /// not set, [`Error::Network`] on a timeout or connection failure, the
    /// mapped status error for a non-2xx response, or [`Error::Http`] if the
    /// body does not deserialize into `T`.
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

    /// Sends a request and discards the response body.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] if the key required by `operation` is
    /// not set, [`Error::Network`] on a timeout or connection failure, or the
    /// mapped status error for a non-2xx response.
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

    /// Sends a request and returns the response body as plain text.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] if the key required by `operation` is
    /// not set, [`Error::Network`] on a timeout or connection failure, or the
    /// mapped status error for a non-2xx response.
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

    /// Send a request and return the raw response for streaming consumption.
    ///
    /// Used by Server-Sent Events endpoints such as search summary streaming.
    /// The configured timeout bounds only connection setup here, not the
    /// lifetime of the stream, so callers may read long-running streams safely.
    pub(crate) async fn request_stream(
        &self,
        method: Method,
        path: &str,
        operation: Operation,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> error::Result<reqwest::Response> {
        let url = self
            .config
            .base_url
            .join(path)
            .map_err(|e| Error::Configuration(format!("invalid path '{path}': {e}")))?;

        let api_key = self.config.api_key_for(operation)?;

        let mut req = self.stream_client.request(method, url.as_str());
        req = req.header("Authorization", api_key);
        req = req.header(reqwest::header::ACCEPT, "text/event-stream");

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

        Ok(response)
    }

    /// Returns the configuration this transport was built with.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
