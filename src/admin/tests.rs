//! Tests for the admin/management API endpoints.

use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::config::Config;
use crate::SearchcraftClient;

/// Helper: create a client with all three key types.
fn test_client(base_url: &str) -> SearchcraftClient {
    let config = Config::new(base_url, Some("test-read-key"), Some("test-ingest-key"))
        .unwrap()
        .with_admin_key("test-admin-key");
    SearchcraftClient::from_config(config).unwrap()
}

/// Wraps a payload in the `{ status, data }` envelope every engine endpoint
/// returns.
fn envelope(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "status": 200, "data": data })
}

// ── Index tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn list_indices_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": { "index_names": ["products", "users"] }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.list_indices().await.unwrap();
    assert_eq!(resp.index_names, vec!["products", "users"]);
}

#[tokio::test]
async fn get_index_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": { "language": "en", "search_fields": ["title", "body"] }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_index("products").await.unwrap();
    assert_eq!(resp.language.as_deref(), Some("en"));
    assert_eq!(
        resp.search_fields.as_deref(),
        Some(vec!["title".to_string(), "body".to_string()].as_slice())
    );
}

#[tokio::test]
async fn delete_index_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": "index deleted"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_index("products").await.unwrap();
    assert_eq!(resp, "index deleted");
}

#[tokio::test]
async fn get_index_stats_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/stats"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": { "document_count": 42 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_index_stats("products").await.unwrap();
    assert_eq!(resp.document_count, 42);
}

#[tokio::test]
async fn get_index_capabilities_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/capabilities"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": {
                "ai": {
                    "enabled": true,
                    "searchSummaryConfigured": true,
                    "llmProviderConfigured": true,
                    "llmModelConfigured": false
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_index_capabilities("products").await.unwrap();
    assert!(resp.ai.enabled);
    assert!(resp.ai.search_summary_configured);
    assert!(resp.ai.llm_provider_configured);
    assert!(!resp.ai.llm_model_configured);
}

#[tokio::test]
async fn create_index_posts_to_the_collection_with_the_name_in_the_body() {
    use super::types::{FieldConfig, FieldType, IndexConfig};
    use std::collections::HashMap;

    let server = MockServer::start().await;
    // Creation is POST /index — PUT /index/:index is a replace and 404s for an
    // index that does not exist yet.
    Mock::given(method("POST"))
        .and(path("/index"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!({
            "index": {
                "name": "products",
                "language": "en",
                "fields": {
                    "title": { "type": "text", "stored": true }
                }
            },
            "override_if_exists": false
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("index created."))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let config = IndexConfig {
        language: Some("en".into()),
        fields: Some(HashMap::from([(
            "title".to_string(),
            FieldConfig {
                stored: Some(true),
                ..FieldConfig::new(FieldType::Text)
            },
        )])),
        ..Default::default()
    };

    let resp = client.create_index("products", &config).await.unwrap();
    assert_eq!(resp, "index created.");
}

#[tokio::test]
async fn create_index_overwriting_sets_the_override_flag() {
    use super::types::IndexConfig;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index"))
        .and(body_json(serde_json::json!({
            "index": { "name": "products" },
            "override_if_exists": true
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("index created."))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .create_index_overwriting("products", &IndexConfig::default())
        .await
        .unwrap();
    assert_eq!(resp, "index created.");
}

#[tokio::test]
async fn replace_index_puts_to_the_named_index() {
    use super::types::IndexConfig;

    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/index/products"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!({
            "index": { "name": "products", "language": "en" },
            "override_if_exists": true
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("index schema replaced"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let config = IndexConfig {
        language: Some("en".into()),
        ..Default::default()
    };
    let resp = client.replace_index("products", &config).await.unwrap();
    assert_eq!(resp, "index schema replaced");
}

#[tokio::test]
async fn update_index_sends_a_flat_body_with_only_changed_fields() {
    use super::types::IndexConfig;

    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/index/products"))
        .and(header("Authorization", "test-ingest-key"))
        // The patch endpoint takes the config at the top level, unlike create
        // and replace which nest it under `index`. Unset fields are omitted so
        // the engine leaves them alone.
        .and(body_json(serde_json::json!({ "auto_commit_delay": 5000 })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("index updated"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let config = IndexConfig {
        auto_commit_delay: Some(5000),
        ..Default::default()
    };

    let resp = client.update_index("products", &config).await.unwrap();
    assert_eq!(resp, "index updated");
}

#[tokio::test]
async fn get_all_index_stats_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/stats"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": {
                "index_count": 2,
                "indices": [
                    { "products": { "document_count": 10 } },
                    { "users": { "document_count": 5 } }
                ],
                "total_document_count": 15
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_all_index_stats().await.unwrap();
    assert_eq!(resp.index_count, 2);
    assert_eq!(resp.total_document_count, 15);
    assert_eq!(resp.indices[0]["products"].document_count, 10);
}

// ── Document tests ───────────────────────────────────────────────────

#[tokio::test]
async fn insert_document_sends_array_body() {
    let server = MockServer::start().await;
    let doc = serde_json::json!({"id": "doc-1", "title": "Test"});
    Mock::given(method("POST"))
        .and(path("/index/products/documents"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(
            serde_json::json!([{"id": "doc-1", "title": "Test"}]),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("ok"))))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.insert_document("products", &doc).await.unwrap();
    assert_eq!(resp, "ok");
}

#[tokio::test]
async fn delete_document_sends_query_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/documents/query"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "detail": "deleted", "num_removed": 1
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_document("products", "doc-1").await.unwrap();
    assert_eq!(resp.num_removed, 1);
}

#[tokio::test]
async fn batch_insert_documents_sends_array_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index/products/documents"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!([
            {"id": "doc-1", "title": "One"},
            {"id": "doc-2", "title": "Two"}
        ])))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("2 documents added"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let docs = vec![
        serde_json::json!({"id": "doc-1", "title": "One"}),
        serde_json::json!({"id": "doc-2", "title": "Two"}),
    ];
    let resp = client
        .batch_insert_documents("products", &docs)
        .await
        .unwrap();
    assert_eq!(resp, "2 documents added");
}

#[tokio::test]
async fn batch_delete_documents_sends_id_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/documents"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!({ "id": ["doc-1", "doc-2"] })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "detail": "deleted", "num_removed": 2
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .batch_delete_documents("products", &["doc-1", "doc-2"])
        .await
        .unwrap();
    assert_eq!(resp.num_removed, 2);
}

#[tokio::test]
async fn delete_all_documents_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/documents/all"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("all documents deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_all_documents("products").await.unwrap();
    assert_eq!(resp, "all documents deleted");
}

#[tokio::test]
async fn get_document_deserializes_into_the_requested_type() {
    #[derive(serde::Deserialize)]
    struct Product {
        title: String,
        price: u32,
    }

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/documents/internal-1"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "doc": {"title": "Laptop", "price": 999},
                "document_id": "internal-1",
                "score": 1.0,
                "source_index": "products"
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let hit = client
        .get_document::<Product>("products", "internal-1")
        .await
        .unwrap();
    assert_eq!(hit.document_id, "internal-1");
    assert_eq!(hit.doc.title, "Laptop");
    assert_eq!(hit.doc.price, 999);
}

// ── Federation tests ─────────────────────────────────────────────────

#[tokio::test]
async fn list_federations_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/federation"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([
                {
                    "name": "global",
                    "friendly_name": "Global",
                    "created_at": "2024-01-01",
                    "created_by": "admin",
                    "last_modified": "2024-01-01",
                    "last_modified_by": "admin",
                    "organization_id": "org-1",
                    "index_configurations": []
                }
            ]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.list_federations().await.unwrap();
    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].name, "global");
}

#[tokio::test]
async fn get_federation_stats_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/federation/global/stats"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "federation_name": "global",
                "num_docs": 100,
                "indices": [{"index_name": "products", "num_docs": 100, "space_usage": 1024}]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_federation_stats("global").await.unwrap();
    assert_eq!(resp.num_docs, 100);
    assert_eq!(resp.indices.len(), 1);
}

#[tokio::test]
async fn get_federation_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/federation/global"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "name": "global",
                "friendly_name": "Global",
                "created_at": "2024-01-01",
                "created_by": "admin",
                "last_modified": "2024-01-02",
                "last_modified_by": "admin",
                "organization_id": "org-1",
                "index_configurations": [
                    { "name": "products", "weight_multiplier": 1.5 }
                ]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_federation("global").await.unwrap();
    assert_eq!(resp.friendly_name, "Global");
    assert_eq!(resp.index_configurations.len(), 1);
    assert!((resp.index_configurations[0].weight_multiplier - 1.5).abs() < f32::EPSILON);
}

#[tokio::test]
async fn delete_federation_uses_ingest_key() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/federation/global"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("federation deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_federation("global").await.unwrap();
    assert_eq!(resp, "federation deleted");
}

// ── Auth tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn list_auth_keys_uses_admin_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/key"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([
                {
                    "token": "k1",
                    "name": "read-key",
                    "permissions": 1,
                    "allowed_indexes": ["products"],
                    "status": "active"
                }
            ]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.list_auth_keys().await.unwrap();
    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].token, "k1");
}

#[tokio::test]
async fn create_auth_key_sends_correct_body() {
    use super::types::{AuthKeyPermission, AuthKeyStatus, CreateAuthKeyRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/key"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "token": "new-key",
                "name": "test",
                "permissions": 1,
                "allowed_indexes": ["products"],
                "status": "active"
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let req = CreateAuthKeyRequest {
        allowed_indexes: Some(vec!["products".into()]),
        permissions: AuthKeyPermission::READ,
        name: "test".into(),
        status: AuthKeyStatus::Active,
        organization_id: None,
        organization_name: None,
        application_id: None,
        application_name: None,
        federation_name: None,
    };
    let resp = client.create_auth_key(&req).await.unwrap();
    assert_eq!(resp.token, "new-key");
}

#[tokio::test]
async fn list_index_auth_keys_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/index/products"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([
                {
                    "token": "k1",
                    "name": "products-read",
                    "permissions": 1,
                    "allowed_indexes": ["products"],
                    "status": "active"
                }
            ]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.list_index_auth_keys("products").await.unwrap();
    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].name.as_deref(), Some("products-read"));
}

#[tokio::test]
async fn get_auth_key_sends_correct_request() {
    use super::types::{AuthKeyPermission, AuthKeyStatus};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/key/k1"))
        .and(header("Authorization", "test-admin-key"))
        // The engine answers this route with a list, not a single object.
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([{
                "token": "k1",
                "name": "ingest-key",
                "permissions": 15,
                "allowed_indexes": ["products"],
                "status": "inactive",
                "organization_id": "org-7",
                "federation_name": "global"
            }]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_auth_key("k1").await.unwrap().expect("key found");
    assert_eq!(resp.permissions, AuthKeyPermission::INGEST);
    assert_eq!(resp.status, AuthKeyStatus::Inactive);
    assert_eq!(resp.organization_id.as_deref(), Some("org-7"));
    assert_eq!(resp.federation_name.as_deref(), Some("global"));
}

#[tokio::test]
async fn create_auth_key_constructor_sends_expected_body() {
    use super::types::{AuthKeyPermission, CreateAuthKeyRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/key"))
        // The constructor defaults to an active key and omits unset scoping.
        .and(body_json(serde_json::json!({
            "allowed_indexes": ["products"],
            "permissions": 63,
            "name": "ops",
            "status": "active"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "token": "new-key",
                "name": "ops",
                "permissions": 63,
                "allowed_indexes": ["products"],
                "status": "active"
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let req = CreateAuthKeyRequest::new("ops", AuthKeyPermission::ADMIN, ["products"]);
    let resp = client.create_auth_key(&req).await.unwrap();
    assert_eq!(resp.permissions, AuthKeyPermission::ADMIN);
}

#[tokio::test]
async fn update_auth_key_sends_only_set_fields() {
    use super::types::{AuthKeyStatus, UpdateAuthKeyRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/key/k1"))
        .and(header("Authorization", "test-admin-key"))
        .and(body_json(serde_json::json!({ "status": "inactive" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "token": "k1",
                "name": "read-key",
                "permissions": 1,
                "allowed_indexes": ["products"],
                "status": "inactive"
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let req = UpdateAuthKeyRequest {
        status: Some(AuthKeyStatus::Inactive),
        ..Default::default()
    };
    let resp = client.update_auth_key("k1", &req).await.unwrap();
    assert_eq!(resp.status, AuthKeyStatus::Inactive);
}

#[tokio::test]
async fn delete_auth_key_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/auth/key/k1"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("key deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert_eq!(client.delete_auth_key("k1").await.unwrap(), "key deleted");
}

#[tokio::test]
async fn delete_all_auth_keys_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/auth/key"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("all keys deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert_eq!(
        client.delete_all_auth_keys().await.unwrap(),
        "all keys deleted"
    );
}

#[tokio::test]
async fn list_application_auth_keys_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/application/42"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([]))))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert!(client
        .list_application_auth_keys("42")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn list_organization_auth_keys_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/organization/7"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([]))))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert!(client
        .list_organization_auth_keys("7")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn list_federation_auth_keys_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/federation/global"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([]))))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert!(client
        .list_federation_auth_keys("global")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn auth_requires_admin_key() {
    let client = SearchcraftClient::new("https://example.com", Some("rk"), Some("ik")).unwrap();
    let err = client.list_auth_keys().await.unwrap_err();
    assert!(err.to_string().contains("admin_key"));
}

// ── Health tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn health_check_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/healthcheck"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": "ok"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.health_check().await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.data, "ok");
}

// ── Measure tests ────────────────────────────────────────────────────

#[tokio::test]
async fn measure_status_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/measure/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 200,
            "data": { "enabled": true }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_measure_status().await.unwrap();
    assert!(resp.enabled);
}

#[tokio::test]
async fn measure_dashboard_summary_returns_raw_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/measure/dashboard/summary"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "total_searches": 1234,
                "top_terms": ["laptop", "phone"]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .get_measure_dashboard_summary(&Default::default())
        .await
        .unwrap();
    assert_eq!(resp["total_searches"], 1234);
    assert_eq!(resp["top_terms"][0], "laptop");
}

#[tokio::test]
async fn track_measure_event_sends_expected_body() {
    use super::types::{event_names, MeasureEvent, MeasureRequestProperties, MeasureRequestUser};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/measure/event"))
        .and(header("Authorization", "test-ingest-key"))
        // Unset optional properties must not appear on the wire.
        .and(body_json(serde_json::json!({
            "event_name": "document_clicked",
            "properties": {
                "searchcraft_index_names": ["products"],
                "external_document_id": "doc-1",
                "document_position": 3
            },
            "user": { "user_id": "user-42" }
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("event recorded"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let event = MeasureEvent::new(
        event_names::DOCUMENT_CLICKED,
        MeasureRequestProperties {
            external_document_id: Some("doc-1".into()),
            document_position: Some(3),
            ..MeasureRequestProperties::new(["products"])
        },
        MeasureRequestUser::new("user-42"),
    );

    let resp = client.track_measure_event(&event).await.unwrap();
    assert_eq!(resp, "event recorded");
}

#[tokio::test]
async fn track_measure_batch_wraps_events_in_items() {
    use super::types::{event_names, MeasureEvent, MeasureRequestProperties, MeasureRequestUser};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/measure/batch"))
        .and(header("Authorization", "test-ingest-key"))
        // The batch endpoint expects `{ "items": [...] }`, not a bare array.
        .and(body_json(serde_json::json!({
            "items": [
                {
                    "event_name": "search_requested",
                    "properties": { "searchcraft_index_names": ["products"] },
                    "user": { "user_id": "u-1" }
                },
                {
                    "event_name": "search_completed",
                    "properties": { "searchcraft_index_names": ["products"] },
                    "user": { "user_id": "u-1" }
                }
            ]
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("2 events recorded"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let events = vec![
        MeasureEvent::new(
            event_names::SEARCH_REQUESTED,
            MeasureRequestProperties::new(["products"]),
            MeasureRequestUser::new("u-1"),
        ),
        MeasureEvent::new(
            event_names::SEARCH_COMPLETED,
            MeasureRequestProperties::new(["products"]),
            MeasureRequestUser::new("u-1"),
        ),
    ];

    let resp = client.track_measure_batch(&events).await.unwrap();
    assert_eq!(resp, "2 events recorded");
}

// ── Transaction tests ────────────────────────────────────────────────

#[tokio::test]
async fn commit_transaction_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index/products/commit"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("committed"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.commit_transaction("products").await.unwrap();
    assert_eq!(resp, "committed");
}

#[tokio::test]
async fn rollback_transaction_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index/products/rollback"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("rolled back"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.rollback_transaction("products").await.unwrap();
    assert_eq!(resp, "rolled back");
}

// ── Stopwords tests ──────────────────────────────────────────────────

#[tokio::test]
async fn get_stopwords_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/stopwords"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!(["the", "a", "is"]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_stopwords("products").await.unwrap();
    assert_eq!(resp, vec!["the", "a", "is"]);
}

#[tokio::test]
async fn add_stopwords_accepts_str_slices() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index/products/stopwords"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!(["the", "and"])))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("stopwords added"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .add_stopwords("products", &["the", "and"])
        .await
        .unwrap();
    assert_eq!(resp, "stopwords added");
}

#[tokio::test]
async fn delete_stopwords_accepts_owned_strings() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/stopwords"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!(["the"])))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("stopwords deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let words = vec!["the".to_string()];
    let resp = client.delete_stopwords("products", &words).await.unwrap();
    assert_eq!(resp, "stopwords deleted");
}

#[tokio::test]
async fn delete_all_stopwords_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/stopwords/all"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("all stopwords deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_all_stopwords("products").await.unwrap();
    assert_eq!(resp, "all stopwords deleted");
}

// ── Synonyms tests ──────────────────────────────────────────────────

#[tokio::test]
async fn get_synonyms_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/synonyms"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({
                "nyc": ["new york city"]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_synonyms("products").await.unwrap();
    assert_eq!(resp.get("nyc").unwrap(), &vec!["new york city".to_string()]);
}

#[tokio::test]
async fn add_synonyms_sends_array_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index/products/synonyms"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!(["notebook:laptop"])))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("synonyms added"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .add_synonyms("products", &["notebook:laptop"])
        .await
        .unwrap();
    assert_eq!(resp, "synonyms added");
}

#[tokio::test]
async fn delete_synonyms_sends_array_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/synonyms"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!(["notebook:laptop"])))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("synonyms deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .delete_synonyms("products", &["notebook:laptop"])
        .await
        .unwrap();
    assert_eq!(resp, "synonyms deleted");
}

#[tokio::test]
async fn delete_all_synonyms_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/synonyms/all"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("all synonyms deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_all_synonyms("products").await.unwrap();
    assert_eq!(resp, "all synonyms deleted");
}

// ── Endpoints added from the engine review ───────────────────────────

#[tokio::test]
async fn get_default_stopwords_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/stopwords/default"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!(["the", "of", "and"]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_default_stopwords("products").await.unwrap();
    assert_eq!(resp, vec!["the", "of", "and"]);
}

#[tokio::test]
async fn delete_document_by_internal_id_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/index/products/documents/12345"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!("deleted"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client
        .delete_document_by_internal_id("products", "12345")
        .await
        .unwrap();
    assert_eq!(resp, "deleted");
}

#[tokio::test]
async fn create_federation_sends_correct_body() {
    use super::types::FederationRequest;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/federation"))
        .and(header("Authorization", "test-ingest-key"))
        .and(body_json(serde_json::json!({
            "name": "global",
            "friendly_name": "Global",
            "index_configurations": [
                { "name": "products", "weight_multiplier": 1.0 },
                { "name": "articles", "weight_multiplier": 1.0 }
            ]
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("federation created"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let request = FederationRequest::new("global", "Global", ["products", "articles"]);
    let resp = client.create_federation(&request).await.unwrap();
    assert_eq!(resp, serde_json::json!("federation created"));
}

#[tokio::test]
async fn update_federation_sends_correct_request() {
    use super::types::FederationRequest;

    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/federation/global"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!("federation updated"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let request = FederationRequest::new("global", "Global", ["products"]);
    client.update_federation("global", &request).await.unwrap();
}

#[tokio::test]
async fn list_federation_indices_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/federation/global/indices"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!(["products", "articles"]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.list_federation_indices("global").await.unwrap();
    assert_eq!(resp, vec!["products", "articles"]);
}

#[tokio::test]
async fn list_federations_by_organization_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/federation/organization/org-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!([]))))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert!(client
        .list_federations_by_organization("org-1")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn check_auth_key_permissions_maps_403_to_false() {
    use super::types::permissions;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/key/check/1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!(
                "Key has sufficient permissions."
            ))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/auth/key/check/256"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            serde_json::json!({ "status": 403, "data": "Key has insufficient permissions." }),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    assert!(client
        .check_auth_key_permissions(permissions::SEARCH)
        .await
        .unwrap());
    assert!(!client
        .check_auth_key_permissions(permissions::READ_ANALYTICS)
        .await
        .unwrap());
}

#[tokio::test]
async fn measure_dashboard_conversion_and_usage_send_correct_requests() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/measure/dashboard/conversion"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!({ "conversions": 12 }))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/measure/dashboard/usage"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(envelope(serde_json::json!({ "searches": 340 }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let conversion = client
        .get_measure_dashboard_conversion(&Default::default())
        .await
        .unwrap();
    assert_eq!(conversion["conversions"], 12);
    let usage = client
        .get_measure_dashboard_usage(&Default::default())
        .await
        .unwrap();
    assert_eq!(usage["searches"], 340);
}

#[test]
fn dashboard_params_serialize_to_a_query_string() {
    use super::types::{MeasureDashboardParams, MeasureQueryGranularity, MeasureUserType};

    assert_eq!(MeasureDashboardParams::default().to_query_string(), "");

    let params = MeasureDashboardParams {
        organization_id: Some("org 1".into()),
        index_names: vec!["products".into(), "articles".into()],
        user_type: Some(MeasureUserType::Authenticated),
        date_start: Some(1_700_000_000),
        granularity: Some(MeasureQueryGranularity::Days),
        page: Some(0),
        ..Default::default()
    };

    let query = params.to_query_string();
    assert!(query.starts_with('?'));
    // The engine parses index_names as a pipe-delimited list.
    assert!(
        query.contains("index_names=products%7Carticles"),
        "got: {query}"
    );
    assert!(query.contains("organization_id=org%201"), "got: {query}");
    assert!(query.contains("user_type=authenticated"), "got: {query}");
    assert!(query.contains("date_start=1700000000"), "got: {query}");
    assert!(query.contains("granularity=days"), "got: {query}");
    // page 0 is meaningful and must survive serialization
    assert!(query.contains("page=0"), "got: {query}");
}

#[tokio::test]
async fn dashboard_request_carries_the_query_string() {
    use super::types::MeasureDashboardParams;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/measure/dashboard/summary"))
        .and(wiremock::matchers::query_param("organization_id", "org-1"))
        .and(wiremock::matchers::query_param(
            "index_names",
            "products|articles",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(serde_json::json!({}))))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let params = MeasureDashboardParams {
        organization_id: Some("org-1".into()),
        index_names: vec!["products".into(), "articles".into()],
        ..Default::default()
    };
    client.get_measure_dashboard_summary(&params).await.unwrap();
}

// ── Error handling tests ─────────────────────────────────────────────

#[tokio::test]
async fn admin_endpoint_handles_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(envelope(serde_json::json!({"message": "unauthorized"}))),
        )
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.list_indices().await.unwrap_err();
    assert!(matches!(err, crate::error::Error::Authentication { .. }));
}

#[tokio::test]
async fn admin_endpoint_handles_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/nonexistent"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(envelope(serde_json::json!({"message": "index not found"}))),
        )
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let err = client.get_index("nonexistent").await.unwrap_err();
    assert!(matches!(err, crate::error::Error::NotFound(_)));
}

// ── Type serialization tests ─────────────────────────────────────────

#[test]
fn index_config_serde_roundtrip() {
    use super::types::IndexConfig;
    let config = IndexConfig {
        language: Some("en".into()),
        search_fields: Some(vec!["title".into()]),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: IndexConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, parsed);
}

#[test]
fn auth_key_status_serde() {
    use super::types::AuthKeyStatus;
    let json = serde_json::to_string(&AuthKeyStatus::Active).unwrap();
    assert_eq!(json, r#""active""#);
    let parsed: AuthKeyStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, AuthKeyStatus::Active);
}

#[test]
fn field_type_serde_covers_every_variant() {
    use super::types::FieldType;

    // The engine's FieldDeclaration defines exactly these eight types.
    let cases = [
        (FieldType::Text, r#""text""#),
        (FieldType::Facet, r#""facet""#),
        (FieldType::Bool, r#""bool""#),
        (FieldType::F64, r#""f64""#),
        (FieldType::U64, r#""u64""#),
        (FieldType::I64, r#""i64""#),
        (FieldType::Datetime, r#""datetime""#),
        (FieldType::Json, r#""json""#),
    ];

    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected, "serializing {variant:?}");
        let parsed: FieldType = serde_json::from_str(expected).unwrap();
        assert_eq!(parsed, variant, "deserializing {expected}");
    }

    // The engine has no bytes field type.
    assert!(serde_json::from_str::<FieldType>(r#""bytes""#).is_err());
}

#[test]
fn auth_key_permission_is_a_bitmask() {
    use super::types::{permissions, AuthKeyPermission};

    // The conventional presets map onto the masks operators recognise.
    for (preset, wire) in [
        (AuthKeyPermission::READ, 1u64),
        (AuthKeyPermission::INGEST, 15),
        (AuthKeyPermission::ADMIN, 63),
    ] {
        assert_eq!(preset.bits(), wire);
        assert_eq!(serde_json::to_string(&preset).unwrap(), wire.to_string());
        let parsed: AuthKeyPermission = serde_json::from_str(&wire.to_string()).unwrap();
        assert_eq!(parsed, preset);
    }

    // A super-user mask exceeds a byte, which an earlier `u8` representation
    // could not deserialize at all.
    let super_user = AuthKeyPermission::SUPER_USER;
    assert!(super_user.bits() > u64::from(u8::MAX));
    assert!(super_user.contains(permissions::READ_ANALYTICS));
    let parsed: AuthKeyPermission = serde_json::from_str(&super_user.bits().to_string()).unwrap();
    assert_eq!(parsed, super_user);

    // Masks the client does not name still round-trip.
    let parsed: AuthKeyPermission = serde_json::from_str("31").unwrap();
    assert_eq!(parsed.bits(), 31);
    assert_eq!(serde_json::to_string(&parsed).unwrap(), "31");
}

#[test]
fn auth_key_permission_bit_tests() {
    use super::types::{permissions, AuthKeyPermission};

    let ingest = AuthKeyPermission::INGEST;
    assert!(ingest.contains(permissions::MODIFY_DOCUMENTS));
    assert!(ingest.contains(permissions::SEARCH | permissions::MODIFY_SYNONYMS));
    assert!(!ingest.contains(permissions::MODIFY_AUTH));
    assert!(ingest.intersects(permissions::SEARCH | permissions::MODIFY_AUTH));

    let combined =
        AuthKeyPermission::READ | AuthKeyPermission::from_bits(permissions::READ_ANALYTICS);
    assert!(combined.contains(permissions::READ_ANALYTICS));
    assert!(combined.contains(permissions::SEARCH));
    assert!(!combined.contains(permissions::MODIFY_DOCUMENTS));
}

#[test]
fn auth_key_deserializes_engine_payload() {
    use super::types::{AuthKey, AuthKeyPermission, AuthKeyStatus};

    // Mirrors the engine's AuthKeyData: `token`, nullable name and
    // allowed_indexes, and string-valued organization/application ids.
    let key: AuthKey = serde_json::from_value(serde_json::json!({
        "token": "sc-abc",
        "created": "2026-01-01T00:00:00Z",
        "name": null,
        "permissions": 511,
        "allowed_indexes": null,
        "status": "active",
        "organization_id": "org-1",
        "application_id": "app-9"
    }))
    .unwrap();

    assert_eq!(key.token, "sc-abc");
    assert!(key.name.is_none());
    // A null allowed_indexes means the key may reach every index.
    assert!(key.allowed_indexes.is_none());
    assert_eq!(key.permissions, AuthKeyPermission::SUPER_USER);
    assert_eq!(key.status, AuthKeyStatus::Active);
    assert_eq!(key.organization_id.as_deref(), Some("org-1"));
    assert_eq!(key.application_id.as_deref(), Some("app-9"));
}

#[test]
fn measure_user_type_serde() {
    use super::types::MeasureUserType;

    assert_eq!(
        serde_json::to_string(&MeasureUserType::Anonymous).unwrap(),
        r#""anonymous""#
    );
    let parsed: MeasureUserType = serde_json::from_str(r#""authenticated""#).unwrap();
    assert_eq!(parsed, MeasureUserType::Authenticated);
}

#[test]
fn field_config_new_leaves_options_unset() {
    use super::types::{FieldConfig, FieldType};

    let config = FieldConfig::new(FieldType::Datetime);
    let json = serde_json::to_value(&config).unwrap();
    // Only the type is sent, so the engine applies its own defaults.
    assert_eq!(json, serde_json::json!({ "type": "datetime" }));

    let overridden = FieldConfig {
        fast: Some(true),
        precision: Some("seconds".into()),
        ..FieldConfig::new(FieldType::Datetime)
    };
    let json = serde_json::to_value(&overridden).unwrap();
    assert_eq!(json["fast"], true);
    assert_eq!(json["precision"], "seconds");
}

#[test]
fn json_field_options_serialize() {
    use super::types::{FieldConfig, FieldType};

    let config = FieldConfig {
        tokenizer: Some("lowercase".into()),
        expand_dots: Some(false),
        ..FieldConfig::new(FieldType::Json)
    };
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["type"], "json");
    assert_eq!(json["tokenizer"], "lowercase");
    assert_eq!(json["expand_dots"], false);
}

#[test]
fn api_response_envelope_deserializes() {
    use super::types::{ApiResponse, IndexStats};

    let resp: ApiResponse<IndexStats> = serde_json::from_value(serde_json::json!({
        "status": 200,
        "data": { "document_count": 7 }
    }))
    .unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.data.document_count, 7);
}

#[test]
fn measure_constructors_leave_optional_fields_unset() {
    use super::types::{MeasureRequestProperties, MeasureRequestUser};

    let json = serde_json::to_value(MeasureRequestProperties::new(["products", "users"])).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "searchcraft_index_names": ["products", "users"] })
    );

    let json = serde_json::to_value(MeasureRequestUser::new("u-1")).unwrap();
    assert_eq!(json, serde_json::json!({ "user_id": "u-1" }));
}

#[test]
fn create_auth_key_request_constructor_defaults_to_active() {
    use super::types::{AuthKeyPermission, AuthKeyStatus, CreateAuthKeyRequest};

    let req = CreateAuthKeyRequest::new("ops", AuthKeyPermission::ADMIN, ["products", "users"]);
    assert_eq!(req.status, AuthKeyStatus::Active);
    assert_eq!(
        req.allowed_indexes.as_deref(),
        Some(["products".to_string(), "users".to_string()].as_slice())
    );
    assert!(req.organization_id.is_none());
}

#[test]
fn synonyms_map_deserializes() {
    use super::types::SynonymsMap;

    let map: SynonymsMap = serde_json::from_value(serde_json::json!({
        "nyc": ["new york city", "new york"]
    }))
    .unwrap();
    assert_eq!(map["nyc"].len(), 2);
}

#[test]
fn llm_provider_serde_keeps_unknown_providers() {
    use super::types::LlmProvider;

    let json = serde_json::to_string(&LlmProvider::Anthropic).unwrap();
    assert_eq!(json, r#""anthropic""#);
    let parsed: LlmProvider = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, LlmProvider::Anthropic);

    // A provider added by a newer engine must not fail to deserialize.
    let parsed: LlmProvider = serde_json::from_str(r#""some-future-provider""#).unwrap();
    assert_eq!(parsed, LlmProvider::Other("some-future-provider".into()));
    assert_eq!(
        serde_json::to_string(&parsed).unwrap(),
        r#""some-future-provider""#
    );
}

#[test]
fn index_config_ai_roundtrip() {
    use super::types::{
        AiConfig, IndexConfig, LlmProvider, PromptInstruction, SearchSummaryConfig,
    };

    let config = IndexConfig {
        language: Some("en".into()),
        ai_enabled: Some(true),
        ai: Some(AiConfig {
            search_summary: Some(SearchSummaryConfig {
                model: "claude-sonnet-5".into(),
                role: Some("product expert".into()),
                character_limit: Some(500),
                temperature: Some(0.2),
                additional_prompt_instructions: Some(vec![PromptInstruction {
                    custom_instruction: "Be concise.".into(),
                    order: 1,
                }]),
                ..Default::default()
            }),
            llm_provider: LlmProvider::Anthropic,
            llm_region: None,
            llm_base_url: None,
            llm_api_key: None,
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["ai_enabled"], true);
    assert_eq!(json["ai"]["llm_provider"], "anthropic");
    assert_eq!(json["ai"]["search_summary"]["model"], "claude-sonnet-5");
    // Unset optional fields stay off the wire.
    assert!(json["ai"].get("llm_region").is_none());

    let parsed: IndexConfig = serde_json::from_value(json).unwrap();
    assert_eq!(config, parsed);
}

#[test]
fn index_config_without_ai_omits_the_fields() {
    use super::types::IndexConfig;

    let json = serde_json::to_value(IndexConfig::default()).unwrap();
    assert!(json.get("ai").is_none());
    assert!(json.get("ai_enabled").is_none());
}

#[test]
fn measure_event_serializes_new_properties() {
    use super::types::{
        event_names, MeasureEvent, MeasureRequestProperties, MeasureRequestUser, MeasureUserType,
    };

    let event = MeasureEvent {
        event_name: event_names::API_SUMMARY_REQUESTED.into(),
        properties: MeasureRequestProperties {
            searchcraft_organization_id: None,
            searchcraft_application_id: None,
            searchcraft_index_names: vec!["products".into()],
            searchcraft_federation_name: Some("global".into()),
            search_term: Some("laptop".into()),
            search_kind: Some("fuzzy".into()),
            ai_provider: Some("anthropic".into()),
            number_of_documents: None,
            external_document_id: None,
            document_position: None,
            session_id: None,
        },
        user: MeasureRequestUser {
            user_id: "u-1".into(),
            user_type: Some(MeasureUserType::Anonymous),
            country: None,
            city: None,
            device_id: None,
            client_ip: None,
            locale: None,
            os: None,
            platform: None,
            region: None,
            sdk_name: None,
            sdk_version: None,
            user_agent: None,
            latitude: None,
            longitude: None,
        },
    };

    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event_name"], "api_summary_requested");
    assert_eq!(json["properties"]["searchcraft_federation_name"], "global");
    assert_eq!(json["properties"]["search_kind"], "fuzzy");
    assert_eq!(json["properties"]["ai_provider"], "anthropic");
    assert_eq!(json["user"]["user_type"], "anonymous");
}
