//! Document management endpoints.

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
    pub async fn insert_document(
        &self,
        index_name: &str,
        document: &serde_json::Value,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/documents");
        let body = vec![document];
        self.transport
            .request_text(Method::POST, &path, Operation::Write, Some(&body))
            .await
    }

    /// Batch inserts multiple documents into an index.
    ///
    /// `POST /index/{index_name}/documents`
    pub async fn batch_insert_documents(
        &self,
        index_name: &str,
        documents: &[serde_json::Value],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/documents");
        self.transport
            .request_text(Method::POST, &path, Operation::Write, Some(&documents))
            .await
    }

    /// Deletes a document by its source ID using a query.
    ///
    /// `DELETE /index/{index_name}/documents/query`
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
            .request(Method::DELETE, &path, Operation::Write, Some(&body))
            .await
    }

    /// Batch deletes documents by their source IDs.
    ///
    /// `DELETE /index/{index_name}/documents`
    pub async fn batch_delete_documents(
        &self,
        index_name: &str,
        document_ids: &[impl Serialize],
    ) -> error::Result<DocumentDeleteResponse> {
        let path = format!("index/{index_name}/documents");
        let body = serde_json::json!({ "id": document_ids });
        self.transport
            .request(Method::DELETE, &path, Operation::Write, Some(&body))
            .await
    }

    /// Deletes all documents from an index.
    ///
    /// `DELETE /index/{index_name}/documents/all`
    pub async fn delete_all_documents(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/documents/all");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Gets a document by its internal Searchcraft ID.
    ///
    /// `GET /index/{index_name}/documents/{internal_id}`
    pub async fn get_document<T: serde::de::DeserializeOwned>(
        &self,
        index_name: &str,
        internal_id: &str,
    ) -> error::Result<SearchHit<T>> {
        let path = format!("index/{index_name}/documents/{internal_id}");
        self.transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }
}
