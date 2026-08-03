//! Health check endpoint.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::HealthCheckResponse;

impl SearchcraftClient {
    /// Checks that the Searchcraft instance is reachable and healthy.
    ///
    /// A healthy node reports `"ok"` in
    /// [`HealthCheckResponse::data`](super::types::HealthCheckResponse::data).
    ///
    /// `GET /healthcheck`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::Network`](crate::Error::Network) if the
    /// instance is unreachable.
    pub async fn health_check(&self) -> error::Result<HealthCheckResponse> {
        self.transport
            .request(Method::GET, "healthcheck", Operation::Read, None::<&()>)
            .await
    }
}
