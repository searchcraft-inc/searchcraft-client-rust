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
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
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
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "detail": "deleted", "num_removed": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.delete_document("products", "doc-1").await.unwrap();
    assert_eq!(resp.num_removed, 1);
}

// ── Federation tests ─────────────────────────────────────────────────

#[tokio::test]
async fn list_federations_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/federation"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
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
        ])))
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
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "federation_name": "global",
            "num_docs": 100,
            "indices": [{"index_name": "products", "num_docs": 100, "space_usage": 1024}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_federation_stats("global").await.unwrap();
    assert_eq!(resp.num_docs, 100);
    assert_eq!(resp.indices.len(), 1);
}

// ── Auth tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn list_auth_keys_uses_admin_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/key"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "key": "k1",
                "name": "read-key",
                "permissions": 1,
                "allowed_indexes": ["products"],
                "status": "active"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.list_auth_keys().await.unwrap();
    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].key, "k1");
}

#[tokio::test]
async fn create_auth_key_sends_correct_body() {
    use super::types::{AuthKeyPermission, AuthKeyStatus, CreateAuthKeyRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/key"))
        .and(header("Authorization", "test-admin-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "key": "new-key",
            "name": "test",
            "permissions": 1,
            "allowed_indexes": ["products"],
            "status": "active"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let req = CreateAuthKeyRequest {
        allowed_indexes: vec!["products".into()],
        permissions: AuthKeyPermission::Read,
        name: "test".into(),
        status: AuthKeyStatus::Active,
        organization_id: None,
        organization_name: None,
        application_id: None,
        application_name: None,
        federation_name: None,
    };
    let resp = client.create_auth_key(&req).await.unwrap();
    assert_eq!(resp.key, "new-key");
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

// ── Transaction tests ────────────────────────────────────────────────

#[tokio::test]
async fn commit_transaction_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/index/products/commit"))
        .and(header("Authorization", "test-ingest-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string("committed"))
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
        .respond_with(ResponseTemplate::new(200).set_body_string("rolled back"))
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
            ResponseTemplate::new(200).set_body_json(serde_json::json!(["the", "a", "is"])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_stopwords("products").await.unwrap();
    assert_eq!(resp, vec!["the", "a", "is"]);
}

// ── Synonyms tests ──────────────────────────────────────────────────

#[tokio::test]
async fn get_synonyms_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index/products/synonyms"))
        .and(header("Authorization", "test-read-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "nyc": ["new york city"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resp = client.get_synonyms("products").await.unwrap();
    assert_eq!(resp.get("nyc").unwrap(), &vec!["new york city".to_string()]);
}

// ── Error handling tests ─────────────────────────────────────────────

#[tokio::test]
async fn admin_endpoint_handles_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"message": "unauthorized"})),
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
                .set_body_json(serde_json::json!({"message": "index not found"})),
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
fn field_type_serde() {
    use super::types::FieldType;
    let json = serde_json::to_string(&FieldType::Text).unwrap();
    assert_eq!(json, r#""text""#);
    let parsed: FieldType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, FieldType::Text);
}
