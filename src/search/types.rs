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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OccurMode {
    /// The clause should match. This is the engine's default.
    ///
    /// A request carrying a single `should` clause is promoted to `must` by
    /// the engine, so it only behaves as "optional" alongside other clauses.
    #[default]
    Should,
    /// The clause must match; documents without it are excluded.
    Must,
    /// The clause must not match; documents containing it are excluded.
    #[serde(rename = "mustnot")]
    MustNot,
}

/// Which fields a `fuzzy` or `term` clause searches.
///
/// Defaults to the index's configured `search_fields` when omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldSelector {
    /// A single field.
    Single(String),
    /// Several fields, weighted equally.
    Multi(Vec<String>),
    /// Several fields, each with its own relevance boost.
    MultiWithBoost(std::collections::HashMap<String, f32>),
}

impl From<&str> for FieldSelector {
    fn from(field: &str) -> Self {
        Self::Single(field.to_string())
    }
}

/// The inner context of a query clause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryContext {
    /// The query string.
    pub ctx: String,
    /// Fields to search. Applies to `fuzzy` and `term` clauses only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<FieldSelector>,
}

impl QueryContext {
    /// Creates a context searching the index's default fields.
    #[must_use]
    pub fn new(ctx: impl Into<String>) -> Self {
        Self {
            ctx: ctx.into(),
            fields: None,
        }
    }
}

/// The context of a `more-like-this` clause, which takes a document ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoreLikeThisContext {
    /// Searchcraft's internal ID of the reference document, as found on
    /// [`SearchHit::document_id`].
    pub ctx: String,
}

/// A single search query clause.
///
/// Serializes to `{ "fuzzy": { "ctx": "..." } }` — or `exact`, `dynamic`,
/// `term`, `more-like-this` — optionally alongside an `occur` field. Exactly
/// one kind should be set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Occur mode for boolean composition. Defaults to `should` server-side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occur: Option<OccurMode>,
    /// Typo-tolerant match. Only searches text fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzzy: Option<QueryContext>,
    /// Exact match, parsed with the engine's query syntax
    /// (e.g. `title:rebel`, `price:[10 TO 100]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact: Option<QueryContext>,
    /// A blend of exact and fuzzy that adapts to the term's length and
    /// contents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<QueryContext>,
    /// Matches a single term against a specific field, without query-syntax
    /// parsing. Useful for programmatic filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<QueryContext>,
    /// Finds documents similar to a reference document.
    #[serde(rename = "more-like-this", skip_serializing_if = "Option::is_none")]
    pub more_like_this: Option<MoreLikeThisContext>,
}

impl SearchQuery {
    /// An empty clause with no kind set.
    fn empty() -> Self {
        Self {
            occur: None,
            fuzzy: None,
            exact: None,
            dynamic: None,
            term: None,
            more_like_this: None,
        }
    }

    /// Create a fuzzy query.
    #[must_use]
    pub fn fuzzy(ctx: impl Into<String>) -> Self {
        Self {
            fuzzy: Some(QueryContext::new(ctx)),
            ..Self::empty()
        }
    }

    /// Create an exact query.
    #[must_use]
    pub fn exact(ctx: impl Into<String>) -> Self {
        Self {
            exact: Some(QueryContext::new(ctx)),
            ..Self::empty()
        }
    }

    /// Create a dynamic query.
    #[must_use]
    pub fn dynamic(ctx: impl Into<String>) -> Self {
        Self {
            dynamic: Some(QueryContext::new(ctx)),
            ..Self::empty()
        }
    }

    /// Create a term query against a single field.
    ///
    /// ```
    /// use searchcraft::search::types::SearchQuery;
    ///
    /// let q = SearchQuery::term("electronics", "category");
    /// assert_eq!(
    ///     serde_json::to_value(&q).unwrap(),
    ///     serde_json::json!({"term": {"ctx": "electronics", "fields": "category"}})
    /// );
    /// ```
    #[must_use]
    pub fn term(ctx: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            term: Some(QueryContext {
                ctx: ctx.into(),
                fields: Some(FieldSelector::Single(field.into())),
            }),
            ..Self::empty()
        }
    }

    /// Create a more-like-this query from a document's internal ID.
    #[must_use]
    pub fn more_like_this(document_id: impl Into<String>) -> Self {
        Self {
            more_like_this: Some(MoreLikeThisContext {
                ctx: document_id.into(),
            }),
            ..Self::empty()
        }
    }

    /// Restrict a fuzzy or term clause to specific fields.
    ///
    /// Has no effect on other query kinds, which the engine does not scope by
    /// field.
    #[must_use]
    pub fn with_fields(mut self, fields: FieldSelector) -> Self {
        if let Some(ctx) = self.fuzzy.as_mut().or(self.term.as_mut()) {
            ctx.fields = Some(fields);
        }
        self
    }

    /// Set the occur mode on this query.
    #[must_use]
    pub fn with_occur(mut self, occur: OccurMode) -> Self {
        self.occur = Some(occur);
        self
    }
}

/// The query payload — either a single query or a list of boolean clauses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
// `Single` is the common case and boxing it would make the ergonomic path
// worse for a struct that is only a few pointers wide.
#[allow(clippy::large_enum_variant)]
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
    /// Maximum number of results to return. Defaults to 20 server-side.
    ///
    /// The engine caps this at its configured maximum (200 unless the operator
    /// raised it) by clamping, not by rejecting the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Offset for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Field to order results by. Ordering by a field means hits come back
    /// with no [`score`](SearchHit::score).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    /// Sort direction. Defaults to descending server-side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<crate::types::SortDirection>,
    /// Datetime field used to decay relevance for older documents, overriding
    /// the index's own setting for this query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_decay_field: Option<String>,
    /// Relevance weight for this index within a federation search. Defaults to
    /// `1.0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_weighting: Option<f32>,
    /// The minimum number of optional (`should`) clauses a document must match.
    ///
    /// When omitted the engine applies its Elasticsearch-compatible default: a
    /// query of only `should` clauses requires 1, and a query containing any
    /// `must` or `mustnot` clause requires 0. The value must not exceed the
    /// number of `should` clauses in the query.
    ///
    /// Added in engine 0.11.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_number_should_match: Option<u32>,
}

/// A single search hit in the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit<T = serde_json::Value> {
    /// The matched document, containing the index's stored fields.
    pub doc: T,
    /// Searchcraft's internal document ID, distinct from your own `id` field.
    ///
    /// Pass this to [`get_document`](crate::SearchcraftClient::get_document).
    pub document_id: String,
    /// The relevance score.
    ///
    /// `None` when the results were ordered by a field rather than by
    /// relevance — an `order_by` query carries no score.
    #[serde(default)]
    pub score: Option<f64>,
    /// The index this hit came from, which matters for federation searches.
    #[serde(default)]
    pub source_index: String,
}

/// A single node in a facet tree, with its document count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacetPath {
    /// The facet path (e.g. `/electronics/laptops`).
    pub path: String,
    /// Number of documents under this path.
    pub count: u64,
    /// Nested facet paths, if the server returned a tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FacetPath>>,
}

/// Facet results keyed by field name.
pub type Facet = std::collections::HashMap<String, Vec<FacetPath>>;

/// The data portion of a search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponseData<T = serde_json::Value> {
    /// The matching documents.
    pub hits: Vec<SearchHit<T>>,
    /// Total number of matching documents.
    pub count: u64,
    /// Time the search took, in seconds.
    pub time_taken: f64,
    /// Facet counts, when the index defines facet fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Facet>>,
}

/// A single event in an AI summary stream.
///
/// A stream yields one [`Metadata`](SummaryStreamEvent::Metadata) event, zero
/// or more [`Delta`](SummaryStreamEvent::Delta) events, and a final
/// [`Done`](SummaryStreamEvent::Done) or [`Error`](SummaryStreamEvent::Error).
/// Malformed frames surface as synthetic `Error` events rather than ending the
/// stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum SummaryStreamEvent {
    /// Sent once, before any content.
    Metadata(SummaryMetadata),
    /// An incremental chunk of generated summary text.
    Delta(SummaryDelta),
    /// Terminal event on success.
    Done(SummaryDone),
    /// Terminal event on failure, or a malformed frame.
    Error(SummaryError),
}

/// Payload of a [`SummaryStreamEvent::Metadata`] event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryMetadata {
    /// Number of search results fed into the summary prompt.
    pub results_count: u64,
    /// Whether the summary was served from cache.
    pub cached: bool,
}

/// Payload of a [`SummaryStreamEvent::Delta`] event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryDelta {
    /// The text chunk. Append these in order to build the full summary.
    pub content: String,
}

/// Payload of a [`SummaryStreamEvent::Done`] event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryDone {
    /// Number of search results the summary was based on.
    pub results_count: u64,
}

/// Payload of a [`SummaryStreamEvent::Error`] event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryError {
    /// Human-readable description of the failure.
    pub message: String,
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
            time_decay_field: None,
            index_weighting: None,
            minimum_number_should_match: None,
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
            time_decay_field: None,
            index_weighting: None,
            minimum_number_should_match: None,
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
            time_decay_field: None,
            index_weighting: None,
            minimum_number_should_match: None,
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
        assert!((resp.data.hits[0].score.unwrap() - 0.95).abs() < f64::EPSILON);
        assert_eq!(resp.data.hits[0].source_index, "products");
    }

    #[test]
    fn search_query_roundtrip() {
        let q = SearchQuery::fuzzy("test query");
        let json = serde_json::to_string(&q).unwrap();
        let parsed: SearchQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(q, parsed);
    }

    #[test]
    fn term_and_more_like_this_queries_serialize() {
        let term = SearchQuery::term("electronics", "category").with_occur(OccurMode::Must);
        assert_eq!(
            serde_json::to_value(&term).unwrap(),
            serde_json::json!({
                "occur": "must",
                "term": { "ctx": "electronics", "fields": "category" }
            })
        );

        // The engine spells this kind kebab-case.
        let mlt = SearchQuery::more_like_this("12345");
        assert_eq!(
            serde_json::to_value(&mlt).unwrap(),
            serde_json::json!({ "more-like-this": { "ctx": "12345" } })
        );
    }

    #[test]
    fn mustnot_occur_serializes_without_a_separator() {
        let q = SearchQuery::exact("draft").with_occur(OccurMode::MustNot);
        assert_eq!(
            serde_json::to_value(&q).unwrap(),
            serde_json::json!({ "occur": "mustnot", "exact": { "ctx": "draft" } })
        );
    }

    #[test]
    fn fuzzy_query_can_target_specific_fields() {
        let single =
            SearchQuery::fuzzy("laptop").with_fields(FieldSelector::Single("title".into()));
        assert_eq!(
            serde_json::to_value(&single).unwrap(),
            serde_json::json!({ "fuzzy": { "ctx": "laptop", "fields": "title" } })
        );

        let multi = SearchQuery::fuzzy("laptop")
            .with_fields(FieldSelector::Multi(vec!["title".into(), "body".into()]));
        assert_eq!(
            serde_json::to_value(&multi).unwrap(),
            serde_json::json!({ "fuzzy": { "ctx": "laptop", "fields": ["title", "body"] } })
        );

        let boosted = SearchQuery::fuzzy("laptop").with_fields(FieldSelector::MultiWithBoost(
            std::collections::HashMap::from([("title".to_string(), 2.0)]),
        ));
        assert_eq!(
            serde_json::to_value(&boosted).unwrap(),
            serde_json::json!({ "fuzzy": { "ctx": "laptop", "fields": { "title": 2.0 } } })
        );
    }

    #[test]
    fn optional_request_fields_are_omitted_until_set() {
        let mut req = SearchRequest {
            query: QueryPayload::Single(SearchQuery::fuzzy("laptop")),
            limit: None,
            offset: None,
            order_by: None,
            sort: None,
            time_decay_field: None,
            index_weighting: None,
            minimum_number_should_match: None,
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "query": { "fuzzy": { "ctx": "laptop" } } })
        );

        req.minimum_number_should_match = Some(2);
        req.time_decay_field = Some("published_at".into());
        req.index_weighting = Some(1.5);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["minimum_number_should_match"], 2);
        assert_eq!(json["time_decay_field"], "published_at");
        assert_eq!(json["index_weighting"], 1.5);
    }

    #[test]
    fn hits_without_a_score_deserialize() {
        // Ordering by a field makes the engine emit a null score.
        let json = serde_json::json!({
            "status": 200,
            "data": {
                "hits": [{
                    "doc": {"title": "Laptop"},
                    "document_id": "doc-1",
                    "score": null,
                    "source_index": "products"
                }],
                "count": 1,
                "time_taken": 0.01
            }
        });

        let resp: SearchResponse = serde_json::from_value(json).unwrap();
        assert!(resp.data.hits[0].score.is_none());
    }

    #[test]
    fn search_response_without_facets_deserializes() {
        let json = serde_json::json!({
            "status": 200,
            "data": { "hits": [], "count": 0, "time_taken": 1.0 }
        });
        let resp: SearchResponse = serde_json::from_value(json).unwrap();
        assert!(resp.data.facets.is_none());
    }

    #[test]
    fn search_response_with_facets_deserializes() {
        let json = serde_json::json!({
            "status": 200,
            "data": {
                "hits": [],
                "count": 0,
                "time_taken": 1.0,
                "facets": [{
                    "category": [{
                        "path": "/electronics",
                        "count": 12,
                        "children": [
                            { "path": "/electronics/laptops", "count": 5 }
                        ]
                    }]
                }]
            }
        });

        let resp: SearchResponse = serde_json::from_value(json).unwrap();
        let facets = resp.data.facets.expect("facets present");
        let category = &facets[0]["category"];
        assert_eq!(category[0].path, "/electronics");
        assert_eq!(category[0].count, 12);

        let children = category[0].children.as_ref().expect("children present");
        assert_eq!(children[0].path, "/electronics/laptops");
        assert_eq!(children[0].count, 5);
        assert!(children[0].children.is_none());
    }

    #[test]
    fn summary_stream_event_uses_type_and_data_tagging() {
        let event = SummaryStreamEvent::Delta(SummaryDelta {
            content: "hello".into(),
        });
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"type": "delta", "data": {"content": "hello"}})
        );

        let parsed: SummaryStreamEvent = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn summary_stream_event_variants_roundtrip() {
        let events = [
            SummaryStreamEvent::Metadata(SummaryMetadata {
                results_count: 3,
                cached: true,
            }),
            SummaryStreamEvent::Delta(SummaryDelta {
                content: "text".into(),
            }),
            SummaryStreamEvent::Done(SummaryDone { results_count: 3 }),
            SummaryStreamEvent::Error(SummaryError {
                message: "boom".into(),
            }),
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let parsed: SummaryStreamEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, event);
        }
    }
}
