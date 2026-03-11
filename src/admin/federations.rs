//! Federation management endpoints.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{Federation, FederationStats};

impl SearchcraftClient {
    /// Lists all federations.
    ///
    /// `GET /federation`
    pub async fn list_federations(&self) -> error::Result<Vec<Federation>> {
        self.transport
            .request(Method::GET, "federation", Operation::Read, None::<&()>)
            .await
    }

    /// Gets the configuration for a specific federation.
    ///
    /// `GET /federation/{federation_name}`
    pub async fn get_federation(&self, federation_name: &str) -> error::Result<Federation> {
        let path = format!("federation/{federation_name}");
        self.transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Deletes a federation.
    ///
    /// `DELETE /federation/{federation_name}`
    pub async fn delete_federation(&self, federation_name: &str) -> error::Result<String> {
        let path = format!("federation/{federation_name}");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Gets statistics for a specific federation.
    ///
    /// `GET /federation/{federation_name}/stats`
    pub async fn get_federation_stats(
        &self,
        federation_name: &str,
    ) -> error::Result<FederationStats> {
        let path = format!("federation/{federation_name}/stats");
        self.transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }
}
