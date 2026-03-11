//! Authentication key management endpoints (self-hosted only).

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{AuthKey, CreateAuthKeyRequest, UpdateAuthKeyRequest};

impl SearchcraftClient {
    /// Lists all authentication keys on the cluster.
    ///
    /// `GET /auth/key`
    pub async fn list_auth_keys(&self) -> error::Result<Vec<AuthKey>> {
        self.transport
            .request(Method::GET, "auth/key", Operation::Admin, None::<&()>)
            .await
    }

    /// Gets an individual authentication key by its value.
    ///
    /// `GET /auth/key/{key}`
    pub async fn get_auth_key(&self, key: &str) -> error::Result<AuthKey> {
        let path = format!("auth/key/{key}");
        self.transport
            .request(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Creates a new authentication key.
    ///
    /// `POST /auth/key`
    pub async fn create_auth_key(&self, request: &CreateAuthKeyRequest) -> error::Result<AuthKey> {
        self.transport
            .request(Method::POST, "auth/key", Operation::Admin, Some(request))
            .await
    }

    /// Updates an existing authentication key.
    ///
    /// `POST /auth/key/{key}`
    pub async fn update_auth_key(
        &self,
        key: &str,
        request: &UpdateAuthKeyRequest,
    ) -> error::Result<AuthKey> {
        let path = format!("auth/key/{key}");
        self.transport
            .request(Method::POST, &path, Operation::Admin, Some(request))
            .await
    }

    /// Deletes an individual authentication key.
    ///
    /// `DELETE /auth/key/{key}`
    pub async fn delete_auth_key(&self, key: &str) -> error::Result<String> {
        let path = format!("auth/key/{key}");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Deletes all authentication keys on the cluster.
    ///
    /// `DELETE /auth/key`
    pub async fn delete_all_auth_keys(&self) -> error::Result<String> {
        self.transport
            .request_text(Method::DELETE, "auth/key", Operation::Admin, None::<&()>)
            .await
    }

    /// Lists all authentication keys for a given application.
    ///
    /// `GET /auth/application/{application_id}`
    pub async fn list_application_auth_keys(
        &self,
        application_id: u64,
    ) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/application/{application_id}");
        self.transport
            .request(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Lists all authentication keys for a given organization.
    ///
    /// `GET /auth/organization/{organization_id}`
    pub async fn list_organization_auth_keys(
        &self,
        organization_id: u64,
    ) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/organization/{organization_id}");
        self.transport
            .request(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Lists all authentication keys for a given federation.
    ///
    /// `GET /auth/federation/{federation_name}`
    pub async fn list_federation_auth_keys(
        &self,
        federation_name: &str,
    ) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/federation/{federation_name}");
        self.transport
            .request(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }
}
