//! Search request and response types for the Searchcraft API.

use serde::{Deserialize, Serialize};

/// The query mode determines how the search engine interprets the query string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryMode {
    /// Fuzzy matching — tolerates typos and partial matches.
    Fuzzy,
    /// Exact matching — requires precise term matches.
    Exact,
    /// Dynamic matching — the engine chooses the best strategy.
    Dynamic,
}

/// The occur mode for boolean query composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OccurMode {
    /// At least one clause should match.
    Should,
    /// All clauses must match.
    Must,
}

/// The inner context of a query clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryContext {
    /// The query string.
    pub ctx: String,
}

/// A single search query clause.
///
/// Serialises to `{ "fuzzy": { "ctx": "..." } }` (or `exact` / `dynamic`),
/// optionally with an `occur` field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Optional occur mode for boolean composition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occur: Option<OccurMode>,
    /// Fuzzy query context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzzy: Option<QueryContext>,
    /// Exact query context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact: Option<QueryContext>,
    /// Dynamic query context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<QueryContext>,
}

impl SearchQuery {
    /// Create a fuzzy query.
    pub fn fuzzy(ctx: impl Into<String>) -> Self {
        Self {
            occur: None,
            fuzzy: Some(QueryContext { ctx: ctx.into() }),
            exact: None,
            dynamic: None,
        }
    }

    /// Create an exact query.
    pub fn exact(ctx: impl Into<String>) -> Self {
        Self {
            occur: None,
            fuzzy: None,
            exact: Some(QueryContext { ctx: ctx.into() }),
            dynamic: None,
        }
    }

    /// Create a dynamic query.
    pub fn dynamic(ctx: impl Into<String>) -> Self {
        Self {
            occur: None,
            fuzzy: None,
            exact: None,
            dynamic: Some(QueryContext { ctx: ctx.into() }),
        }
    }

    /// Set the occur mode on this query.
    #[must_use]
    pub fn with_occur(mut self, occur: OccurMode) -> Self {
        self.occur = Some(occur);
        self
    }
}

/// The query payload — either a single query or a list of boolean clauses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryPayload {
    /// A single query clause.
    Single(SearchQuery),
    /// Multiple boolean query clauses.
    Multiple(Vec<SearchQuery>),
}

impl From<SearchQuery> for QueryPayload {
    fn from(q: SearchQuery) -> Self {
        Self::Single(q)
    }
}

impl From<Vec<SearchQuery>> for QueryPayload {
    fn from(qs: Vec<SearchQuery>) -> Self {
        Self::Multiple(qs)
    }
}

/// A search request sent to the Searchcraft API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// The query to execute.
    pub query: QueryPayload,
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Offset for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Field to order results by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<crate::types::SortDirection>,
}

/// A single search hit in the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit<T = serde_json::Value> {
    /// The matched document.
    pub doc: T,
    /// The document ID.
    pub document_id: String,
    /// The relevance score.
    pub score: f64,
    /// The source index name.
    pub source_index: String,
}

/// The data portion of a search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponseData<T = serde_json::Value> {
    /// The matching documents.
    pub hits: Vec<SearchHit<T>>,
    /// Total number of matching documents.
    pub count: u64,
    /// Time taken in milliseconds.
    pub time_taken: f64,
}

/// The full search response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse<T = serde_json::Value> {
    /// HTTP status code.
    pub status: u16,
    /// The response data containing hits and metadata.
    pub data: SearchResponseData<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_query_serialization() {
        let q = SearchQuery::fuzzy("laptop");
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json, serde_json::json!({"fuzzy": {"ctx": "laptop"}}));
    }

    #[test]
    fn exact_query_serialization() {
        let q = SearchQuery::exact("laptop");
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json, serde_json::json!({"exact": {"ctx": "laptop"}}));
    }

    #[test]
    fn dynamic_query_serialization() {
        let q = SearchQuery::dynamic("laptop");
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json, serde_json::json!({"dynamic": {"ctx": "laptop"}}));
    }

    #[test]
    fn query_with_occur_serialization() {
        let q = SearchQuery::fuzzy("laptop").with_occur(OccurMode::Must);
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"occur": "must", "fuzzy": {"ctx": "laptop"}})
        );
    }

    #[test]
    fn search_request_serialization() {
        let req = SearchRequest {
            query: QueryPayload::Single(SearchQuery::fuzzy("laptop")),
            limit: Some(10),
            offset: Some(0),
            order_by: None,
            sort: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["query"]["fuzzy"]["ctx"], "laptop");
        assert_eq!(json["limit"], 10);
        assert_eq!(json["offset"], 0);
        assert!(json.get("order_by").is_none());
    }

    #[test]
    fn search_request_with_sorting() {
        let req = SearchRequest {
            query: QueryPayload::Single(SearchQuery::exact("test")),
            limit: Some(20),
            offset: None,
            order_by: Some("price".into()),
            sort: Some(crate::types::SortDirection::Asc),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["order_by"], "price");
        assert_eq!(json["sort"], "asc");
    }

    #[test]
    fn multiple_queries_serialization() {
        let queries = vec![
            SearchQuery::fuzzy("laptop").with_occur(OccurMode::Must),
            SearchQuery::exact("gaming").with_occur(OccurMode::Should),
        ];
        let req = SearchRequest {
            query: QueryPayload::Multiple(queries),
            limit: Some(10),
            offset: None,
            order_by: None,
            sort: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        let query_arr = json["query"].as_array().unwrap();
        assert_eq!(query_arr.len(), 2);
        assert_eq!(query_arr[0]["occur"], "must");
        assert_eq!(query_arr[1]["occur"], "should");
    }

    #[test]
    fn search_response_deserialization() {
        let json = serde_json::json!({
            "status": 200,
            "data": {
                "hits": [
                    {
                        "doc": {"title": "Gaming Laptop", "price": 999},
                        "document_id": "doc-1",
                        "score": 0.95,
                        "source_index": "products"
                    }
                ],
                "count": 1,
                "time_taken": 12.5
            }
        });
        let resp: SearchResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.data.count, 1);
        assert_eq!(resp.data.hits.len(), 1);
        assert_eq!(resp.data.hits[0].document_id, "doc-1");
        assert!((resp.data.hits[0].score - 0.95).abs() < f64::EPSILON);
        assert_eq!(resp.data.hits[0].source_index, "products");
    }

    #[test]
    fn search_query_roundtrip() {
        let q = SearchQuery::fuzzy("test query");
        let json = serde_json::to_string(&q).unwrap();
        let parsed: SearchQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(q, parsed);
    }
}
