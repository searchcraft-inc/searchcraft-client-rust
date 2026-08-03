//! Federation management endpoints.
//!
//! A federation groups several indices so they can be searched as one with
//! [`search_federation`](SearchcraftClient::search_federation). Federations are
//! created through the Searchcraft dashboard rather than this API.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{Federation, FederationRequest, FederationStats};

impl SearchcraftClient {
    /// Lists every federation visible to the key.
    ///
    /// `GET /federation`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::Authentication`](crate::Error::Authentication)
    /// if the key is rejected.
    pub async fn list_federations(&self) -> error::Result<Vec<Federation>> {
        self.transport
            .request_data(Method::GET, "federation", Operation::Read, None::<&()>)
            .await
    }

    /// Gets the configuration for a specific federation.
    ///
    /// `GET /federation/{federation_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// federation does not exist.
    pub async fn get_federation(&self, federation_name: &str) -> error::Result<Federation> {
        let path = format!("federation/{federation_name}");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Lists the federations belonging to an organization.
    ///
    /// `GET /federation/organization/{organization_id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// organization does not exist.
    pub async fn list_federations_by_organization(
        &self,
        organization_id: &str,
    ) -> error::Result<Vec<Federation>> {
        let path = format!("federation/organization/{organization_id}");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Lists the index names that make up a federation.
    ///
    /// `GET /federation/{federation_name}/indices`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// federation does not exist.
    pub async fn list_federation_indices(
        &self,
        federation_name: &str,
    ) -> error::Result<Vec<String>> {
        let path = format!("federation/{federation_name}/indices");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Creates a federation.
    ///
    /// `POST /federation`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::Validation`](crate::Error::Validation) if the request is
    /// rejected — including when a federation of that name already exists.
    pub async fn create_federation(
        &self,
        request: &FederationRequest,
    ) -> error::Result<serde_json::Value> {
        self.transport
            .request_data(Method::POST, "federation", Operation::Write, Some(request))
            .await
    }

    /// Updates an existing federation.
    ///
    /// `PUT /federation/{federation_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the federation does not
    /// exist.
    pub async fn update_federation(
        &self,
        federation_name: &str,
        request: &FederationRequest,
    ) -> error::Result<serde_json::Value> {
        let path = format!("federation/{federation_name}");
        self.transport
            .request_data(Method::PUT, &path, Operation::Write, Some(request))
            .await
    }

    /// Deletes a federation.
    ///
    /// The federated indices themselves are left in place.
    ///
    /// `DELETE /federation/{federation_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the federation does not
    /// exist.
    pub async fn delete_federation(&self, federation_name: &str) -> error::Result<String> {
        let path = format!("federation/{federation_name}");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Gets document counts and disk usage for a federation, broken down by
    /// index.
    ///
    /// `GET /federation/{federation_name}/stats`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// federation does not exist.
    pub async fn get_federation_stats(
        &self,
        federation_name: &str,
    ) -> error::Result<FederationStats> {
        let path = format!("federation/{federation_name}/stats");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }
}
