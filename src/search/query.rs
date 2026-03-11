//! Idiomatic Rust query builder for Searchcraft search requests.
//!
//! # Examples
//!
//! ```
//! use searchcraft::search::query::QueryBuilder;
//!
//! // Simple fuzzy search
//! let request = QueryBuilder::fuzzy()
//!     .term("laptop")
//!     .limit(10)
//!     .build_request();
//!
//! // Field query with sorting
//! let request = QueryBuilder::exact()
//!     .field("category", "electronics")
//!     .order_by("price", searchcraft::types::SortDirection::Asc)
//!     .limit(20)
//!     .build_request();
//!
//! // Boolean composition
//! let request = QueryBuilder::fuzzy()
//!     .term("laptop")
//!     .and("gaming")
//!     .not("refurbished")
//!     .build_request();
//! ```

use std::fmt::Write;

use super::types::{OccurMode, QueryMode, QueryPayload, SearchQuery, SearchRequest};
use crate::types::SortDirection;

/// An immutable, chainable builder for constructing Searchcraft search queries.
///
/// Each method returns a new `QueryBuilder` instance, leaving the original
/// unchanged. Call [`build`](QueryBuilder::build) to produce a [`SearchQuery`]
/// or [`build_request`](QueryBuilder::build_request) for a complete
/// [`SearchRequest`].
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    mode: QueryMode,
    occur: Option<OccurMode>,
    parts: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by_field: Option<String>,
    sort: Option<SortDirection>,
}

impl QueryBuilder {
    fn new(mode: QueryMode) -> Self {
        Self {
            mode,
            occur: None,
            parts: Vec::new(),
            limit: None,
            offset: None,
            order_by_field: None,
            sort: None,
        }
    }

    /// Create a fuzzy query builder.
    pub fn fuzzy() -> Self {
        Self::new(QueryMode::Fuzzy)
    }

    /// Create an exact query builder.
    pub fn exact() -> Self {
        Self::new(QueryMode::Exact)
    }

    /// Create a dynamic query builder.
    pub fn dynamic() -> Self {
        Self::new(QueryMode::Dynamic)
    }

    /// Set the occur mode for boolean composition.
    #[must_use]
    pub fn occur(mut self, occur: OccurMode) -> Self {
        self.occur = Some(occur);
        self
    }

    /// Add a search term.
    #[must_use]
    pub fn term(mut self, term: impl Into<String>) -> Self {
        self.parts.push(term.into());
        self
    }

    /// Add a `field:value` query.
    #[must_use]
    pub fn field(mut self, field: &str, value: impl std::fmt::Display) -> Self {
        self.parts.push(format!("{field}:{value}"));
        self
    }

    /// Add a `field:IN [v1 v2 ...]` query.
    #[must_use]
    pub fn field_in<V: std::fmt::Display>(mut self, field: &str, values: &[V]) -> Self {
        let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        self.parts.push(format!("{field}:IN [{}]", vals.join(" ")));
        self
    }

    /// Add a range query: `field:[from TO to]` (inclusive) or `field:{from TO to}` (exclusive).
    #[must_use]
    pub fn range(
        mut self,
        field: &str,
        from: impl std::fmt::Display,
        to: impl std::fmt::Display,
        inclusive: bool,
    ) -> Self {
        let (open, close) = if inclusive { ('[', ']') } else { ('{', '}') };
        self.parts
            .push(format!("{field}:{open}{from} TO {to}{close}"));
        self
    }

    /// Add a comparison query: `field:>value`, `field:<=value`, etc.
    #[must_use]
    pub fn compare(mut self, field: &str, op: CompareOp, value: impl std::fmt::Display) -> Self {
        self.parts.push(format!("{field}:{op}{value}"));
        self
    }

    /// Add an AND clause.
    #[must_use]
    pub fn and(mut self, query: impl Into<String>) -> Self {
        self.parts.push("AND".into());
        self.parts.push(query.into());
        self
    }

    /// Add an OR clause.
    #[must_use]
    pub fn or(mut self, query: impl Into<String>) -> Self {
        self.parts.push("OR".into());
        self.parts.push(query.into());
        self
    }

    /// Add a NOT (exclusion) term.
    #[must_use]
    pub fn not(mut self, term: impl std::fmt::Display) -> Self {
        self.parts.push(format!("-{term}"));
        self
    }

    /// Wrap a sub-query in parentheses for grouping.
    #[must_use]
    pub fn group(mut self, query: impl Into<String>) -> Self {
        self.parts.push(format!("({})", query.into()));
        self
    }

    /// Set the maximum number of results.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the pagination offset.
    #[must_use]
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the field and direction to order results by.
    #[must_use]
    pub fn order_by(mut self, field: impl Into<String>, sort: SortDirection) -> Self {
        self.order_by_field = Some(field.into());
        self.sort = Some(sort);
        self
    }

    /// Build the query string from accumulated parts.
    fn build_query_string(&self) -> String {
        self.parts.join(" ")
    }

    /// Build a [`SearchQuery`] from this builder.
    pub fn build(&self) -> SearchQuery {
        let ctx = self.build_query_string();
        let mut q = match self.mode {
            QueryMode::Fuzzy => SearchQuery::fuzzy(ctx),
            QueryMode::Exact => SearchQuery::exact(ctx),
            QueryMode::Dynamic => SearchQuery::dynamic(ctx),
        };
        if let Some(occur) = self.occur {
            q = q.with_occur(occur);
        }
        q
    }

    /// Build a complete [`SearchRequest`] from this builder.
    pub fn build_request(&self) -> SearchRequest {
        SearchRequest {
            query: QueryPayload::Single(self.build()),
            limit: self.limit,
            offset: self.offset,
            order_by: self.order_by_field.clone(),
            sort: self.sort,
        }
    }
}

/// Comparison operators for [`QueryBuilder::compare`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Greater than (`>`).
    Gt,
    /// Less than (`<`).
    Lt,
    /// Greater than or equal (`>=`).
    Gte,
    /// Less than or equal (`<=`).
    Lte,
}

impl std::fmt::Display for CompareOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gt => f.write_char('>'),
            Self::Lt => f.write_char('<'),
            Self::Gte => f.write_str(">="),
            Self::Lte => f.write_str("<="),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::types::OccurMode;

    #[test]
    fn simple_fuzzy_query() {
        let q = QueryBuilder::fuzzy().term("laptop").build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json, serde_json::json!({"fuzzy": {"ctx": "laptop"}}));
    }

    #[test]
    fn field_query() {
        let q = QueryBuilder::exact()
            .field("category", "electronics")
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"exact": {"ctx": "category:electronics"}})
        );
    }

    #[test]
    fn field_in_query() {
        let q = QueryBuilder::fuzzy()
            .field_in("status", &["active", "pending"])
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"fuzzy": {"ctx": "status:IN [active pending]"}})
        );
    }

    #[test]
    fn range_query_inclusive() {
        let q = QueryBuilder::dynamic()
            .range("price", 10, 100, true)
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"dynamic": {"ctx": "price:[10 TO 100]"}})
        );
    }

    #[test]
    fn range_query_exclusive() {
        let q = QueryBuilder::dynamic()
            .range("price", 10, 100, false)
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"dynamic": {"ctx": "price:{10 TO 100}"}})
        );
    }

    #[test]
    fn compare_query() {
        let q = QueryBuilder::exact()
            .compare("price", CompareOp::Gte, 50)
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json, serde_json::json!({"exact": {"ctx": "price:>=50"}}));
    }

    #[test]
    fn boolean_composition() {
        let q = QueryBuilder::fuzzy()
            .term("laptop")
            .and("gaming")
            .not("refurbished")
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"fuzzy": {"ctx": "laptop AND gaming -refurbished"}})
        );
    }

    #[test]
    fn or_composition() {
        let q = QueryBuilder::fuzzy().term("laptop").or("tablet").build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"fuzzy": {"ctx": "laptop OR tablet"}})
        );
    }

    #[test]
    fn group_query() {
        let q = QueryBuilder::fuzzy()
            .group("laptop OR tablet")
            .and("gaming")
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"fuzzy": {"ctx": "(laptop OR tablet) AND gaming"}})
        );
    }

    #[test]
    fn with_occur() {
        let q = QueryBuilder::fuzzy()
            .term("laptop")
            .occur(OccurMode::Must)
            .build();
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"occur": "must", "fuzzy": {"ctx": "laptop"}})
        );
    }

    #[test]
    fn build_request_with_pagination() {
        let req = QueryBuilder::fuzzy()
            .term("laptop")
            .limit(10)
            .offset(20)
            .order_by("price", SortDirection::Asc)
            .build_request();

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["query"]["fuzzy"]["ctx"], "laptop");
        assert_eq!(json["limit"], 10);
        assert_eq!(json["offset"], 20);
        assert_eq!(json["order_by"], "price");
        assert_eq!(json["sort"], "asc");
    }

    #[test]
    fn compare_op_display() {
        assert_eq!(CompareOp::Gt.to_string(), ">");
        assert_eq!(CompareOp::Lt.to_string(), "<");
        assert_eq!(CompareOp::Gte.to_string(), ">=");
        assert_eq!(CompareOp::Lte.to_string(), "<=");
    }
}
