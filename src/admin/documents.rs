//! Document management endpoints.
//!
//! Every method here needs an ingest key except
//! [`get_document`](SearchcraftClient::get_document), which reads.
//!
//! Writes are buffered by the engine until they are committed, either by the
//! index's `auto_commit_delay` or explicitly via
//! [`commit_transaction`](SearchcraftClient::commit_transaction).

use reqwest::Method;
use serde::Serialize;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;
use crate::search::types::SearchHit;

use super::types::DocumentDeleteResponse;

impl SearchcraftClient {
    /// Inserts a single document into an index.
    ///
    /// The document must include an `id` field. Sends the document wrapped
    /// in a single-element array to `POST /index/{index_name}/documents`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, [`Error::NotFound`](crate::Error::NotFound)
    /// if the index does not exist, or
    /// [`Error::Validation`](crate::Error::Validation) if the document does not
    /// match the index schema.
    pub async fn insert_document(
        &self,
        index_name: &str,
        document: &serde_json::Value,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/documents");
        let body = vec![document];
        self.transport
            .request_data(Method::POST, &path, Operation::Write, Some(&body))
            .await
    }

    /// Batch inserts multiple documents into an index.
    ///
    /// `POST /index/{index_name}/documents`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, [`Error::NotFound`](crate::Error::NotFound)
    /// if the index does not exist, or
    /// [`Error::Validation`](crate::Error::Validation) if any document does not
    /// match the index schema.
    pub async fn batch_insert_documents(
        &self,
        index_name: &str,
        documents: &[serde_json::Value],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/documents");
        self.transport
            .request_data(Method::POST, &path, Operation::Write, Some(&documents))
            .await
    }

    /// Deletes a document by its source ID.
    ///
    /// Issues an exact-match query on the `id` field to
    /// `DELETE /index/{index_name}/documents/query`. Deleting an ID that does
    /// not exist succeeds with a `num_removed` of zero.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_document(
        &self,
        index_name: &str,
        document_id: &str,
    ) -> error::Result<DocumentDeleteResponse> {
        let path = format!("index/{index_name}/documents/query");
        let body = serde_json::json!({
            "query": {
                "exact": {
                    "ctx": format!("id:{document_id}")
                }
            }
        });
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, Some(&body))
            .await
    }

    /// Batch deletes documents by their source IDs.
    ///
    /// `DELETE /index/{index_name}/documents`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn batch_delete_documents(
        &self,
        index_name: &str,
        document_ids: &[impl Serialize],
    ) -> error::Result<DocumentDeleteResponse> {
        let path = format!("index/{index_name}/documents");
        let body = serde_json::json!({ "id": document_ids });
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, Some(&body))
            .await
    }

    /// Deletes a document by Searchcraft's internal ID.
    ///
    /// The internal ID is the `document_id` on a [`SearchHit`], as opposed to
    /// [`delete_document`](Self::delete_document), which matches on your own
    /// `id` field.
    ///
    /// `DELETE /index/{index_name}/documents/{internal_id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_document_by_internal_id(
        &self,
        index_name: &str,
        internal_id: &str,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/documents/{internal_id}");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Deletes every document in an index, leaving the index itself in place.
    ///
    /// `DELETE /index/{index_name}/documents/all`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_all_documents(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/documents/all");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Gets a document by its internal Searchcraft ID.
    ///
    /// The internal ID is the `document_id` on a [`SearchHit`], which is
    /// distinct from your own `id` field.
    ///
    /// `GET /index/{index_name}/documents/{internal_id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, [`Error::NotFound`](crate::Error::NotFound) if the
    /// index or document does not exist, or
    /// [`Error::Http`](crate::Error::Http) if the document does not deserialize
    /// into `T`.
    pub async fn get_document<T: serde::de::DeserializeOwned>(
        &self,
        index_name: &str,
        internal_id: &str,
    ) -> error::Result<SearchHit<T>> {
        let path = format!("index/{index_name}/documents/{internal_id}");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }
}
