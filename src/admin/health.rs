//! Health check endpoint.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::HealthCheckResponse;

impl SearchcraftClient {
    /// Performs a health check on the Searchcraft instance.
    ///
    /// `GET /healthcheck`
    pub async fn health_check(&self) -> error::Result<HealthCheckResponse> {
        self.transport
            .request(Method::GET, "healthcheck", Operation::Read, None::<&()>)
            .await
    }
}
