//! Authentication key management endpoints.
//!
//! These endpoints exist only on self-hosted Searchcraft instances and all
//! require an admin key, set with
//! [`Config::with_admin_key`](crate::Config::with_admin_key). Without one,
//! every method here returns
//! [`Error::Configuration`](crate::Error::Configuration) before sending a
//! request.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{AuthKey, CreateAuthKeyRequest, UpdateAuthKeyRequest};

impl SearchcraftClient {
    /// Lists every authentication key on the cluster.
    ///
    /// `GET /auth/key`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::Authentication`](crate::Error::Authentication) if the key lacks
    /// admin permissions.
    pub async fn list_auth_keys(&self) -> error::Result<Vec<AuthKey>> {
        self.transport
            .request_data(Method::GET, "auth/key", Operation::Admin, None::<&()>)
            .await
    }

    /// Gets an individual authentication key by its value.
    ///
    /// The engine answers with a list, which is empty when no such key exists,
    /// so this returns `None` rather than an error for an unknown key.
    ///
    /// `GET /auth/key/{key}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::Authentication`](crate::Error::Authentication) if the key lacks
    /// admin permissions.
    pub async fn get_auth_key(&self, key: &str) -> error::Result<Option<AuthKey>> {
        let path = format!("auth/key/{key}");
        let keys: Vec<AuthKey> = self
            .transport
            .request_data(Method::GET, &path, Operation::Admin, None::<&()>)
            .await?;
        Ok(keys.into_iter().next())
    }

    /// Checks whether the configured admin key carries all of the given
    /// permission bits.
    ///
    /// `GET /auth/key/check/{permission}`
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// # let config = searchcraft::Config::new("https://x.io", Some("k"), None::<String>)?
    /// #     .with_admin_key("admin");
    /// # let client = searchcraft::SearchcraftClient::from_config(config)?;
    /// use searchcraft::admin::types::permissions;
    ///
    /// if client.check_auth_key_permissions(permissions::READ_ANALYTICS).await? {
    ///     // the key may read the measure dashboard
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured. A key that simply lacks the permission yields
    /// `Ok(false)`, not an error.
    pub async fn check_auth_key_permissions(&self, permissions: u64) -> error::Result<bool> {
        let path = format!("auth/key/check/{permissions}");
        match self
            .transport
            .request_data::<String>(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
        {
            Ok(_) => Ok(true),
            // The engine reports an insufficient key as 403 with a message.
            Err(crate::Error::Authentication { status: 403, .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Creates a new authentication key.
    ///
    /// `POST /auth/key`
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// # let config = searchcraft::Config::new("https://x.io", Some("k"), None::<String>)?
    /// #     .with_admin_key("admin");
    /// # let client = searchcraft::SearchcraftClient::from_config(config)?;
    /// use searchcraft::admin::types::{AuthKeyPermission, CreateAuthKeyRequest};
    ///
    /// let request = CreateAuthKeyRequest::new(
    ///     "web-frontend",
    ///     AuthKeyPermission::READ,
    ///     ["products"],
    /// );
    /// let created = client.create_auth_key(&request).await?;
    /// println!("new key: {}", created.token);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::Validation`](crate::Error::Validation) if the request is
    /// rejected.
    pub async fn create_auth_key(&self, request: &CreateAuthKeyRequest) -> error::Result<AuthKey> {
        self.transport
            .request_data(Method::POST, "auth/key", Operation::Admin, Some(request))
            .await
    }

    /// Updates an existing authentication key.
    ///
    /// Only the fields set on `request` are changed.
    ///
    /// `POST /auth/key/{key}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if no such key exists.
    pub async fn update_auth_key(
        &self,
        key: &str,
        request: &UpdateAuthKeyRequest,
    ) -> error::Result<AuthKey> {
        let path = format!("auth/key/{key}");
        self.transport
            .request_data(Method::POST, &path, Operation::Admin, Some(request))
            .await
    }

    /// Deletes an individual authentication key.
    ///
    /// `DELETE /auth/key/{key}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if no such key exists.
    pub async fn delete_auth_key(&self, key: &str) -> error::Result<String> {
        let path = format!("auth/key/{key}");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Deletes every authentication key on the cluster.
    ///
    /// This revokes all access, including the key making the request.
    ///
    /// `DELETE /auth/key`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::Authentication`](crate::Error::Authentication) if the key lacks
    /// admin permissions.
    pub async fn delete_all_auth_keys(&self) -> error::Result<String> {
        self.transport
            .request_data(Method::DELETE, "auth/key", Operation::Admin, None::<&()>)
            .await
    }

    /// Lists the authentication keys scoped to an application.
    ///
    /// `GET /auth/application/{application_id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the application does not
    /// exist.
    pub async fn list_application_auth_keys(
        &self,
        application_id: &str,
    ) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/application/{application_id}");
        self.transport
            .request_data(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Lists the authentication keys scoped to an organization.
    ///
    /// `GET /auth/organization/{organization_id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the organization does not
    /// exist.
    pub async fn list_organization_auth_keys(
        &self,
        organization_id: &str,
    ) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/organization/{organization_id}");
        self.transport
            .request_data(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Lists the authentication keys scoped to a federation.
    ///
    /// `GET /auth/federation/{federation_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the federation does not
    /// exist.
    pub async fn list_federation_auth_keys(
        &self,
        federation_name: &str,
    ) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/federation/{federation_name}");
        self.transport
            .request_data(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }

    /// Lists the authentication keys scoped to an index.
    ///
    /// `GET /auth/index/{index_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// admin key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn list_index_auth_keys(&self, index_name: &str) -> error::Result<Vec<AuthKey>> {
        let path = format!("auth/index/{index_name}");
        self.transport
            .request_data(Method::GET, &path, Operation::Admin, None::<&()>)
            .await
    }
}
