<img alt="Searchcraft" src="https://raw.githubusercontent.com/searchcraft-inc/searchcraft-client-rust/main/header.png">

<h1 align="center">Searchcraft Rust Client</h1>

<p align="center">
An async Rust client library for the <a href="https://searchcraft.io">Searchcraft</a> API.
</p>

<p align="center">
  <a href="https://docs.searchcraft.io"><strong>Documentation</strong></a> ·
  <a href="https://searchcraft.io"><strong>Website</strong></a> ·
  <a href="https://github.com/searchcraft-inc/searchcraft-issues/issues"><strong>Issues</strong></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/searchcraft">
    <img src="https://img.shields.io/crates/v/searchcraft.svg?logo=rust&style=flat" alt="Crates.io">
  </a>
  <a href="https://docs.rs/searchcraft">
    <img src="https://docs.rs/searchcraft/badge.svg" alt="docs.rs">
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Rust-1.75+-orange.svg?logo=rust&style=flat" alt="Rust">
  </a>
  <a href="https://github.com/searchcraft-inc/searchcraft-client-rust/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=flat" alt="License">
  </a>
</p>

---

## Features

- Fully typed, async API built on [`reqwest`](https://docs.rs/reqwest).
- Composable, immutable query builder.
- Support for all Searchcraft query kinds (fuzzy, exact, dynamic, term, more-like-this).
- Full index, document, federation, synonym, and stopword management.
- Streaming AI search summaries over Server-Sent Events.
- Analytics via the Measure API, including the dashboard reports.
- `rustls` by default, with an optional `native-tls` backend.
- API keys held in [`secrecy`](https://docs.rs/secrecy) wrappers so they are not
  logged by accident.

## Installation

Add the crate to your project:

```sh
cargo add searchcraft
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
searchcraft = "0.1"
```

## Quick start

```rust,no_run
use searchcraft::SearchcraftClient;
use searchcraft::search::query::QueryBuilder;

#[tokio::main]
async fn main() -> Result<(), searchcraft::Error> {
    let client = SearchcraftClient::new(
        "https://my-instance.searchcraft.io",
        Some("sc-read-key"),
        None::<String>,
    )?;

    let request = QueryBuilder::fuzzy()
        .term("laptop")
        .limit(10)
        .build_request();

    let response = client
        .search_index::<serde_json::Value>("products", &request)
        .await?;

    for hit in &response.data.hits {
        // `score` is None when results are ordered by a field rather than
        // by relevance.
        match hit.score {
            Some(score) => println!("{} (score: {score:.2})", hit.document_id),
            None => println!("{}", hit.document_id),
        }
    }

    Ok(())
}
```

## Feature flags

| Feature      | Default | Description                   |
|--------------|---------|-------------------------------|
| `rustls`     | ✅      | Use rustls for TLS            |
| `native-tls` | ❌      | Use the platform's native TLS |

To use native TLS instead of rustls:

```toml
[dependencies]
searchcraft = { version = "0.1", default-features = false, features = ["native-tls"] }
```

## API overview

### Search

Search a single index or a federation of indices:

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
use searchcraft::search::query::QueryBuilder;

// Fuzzy search with boolean composition
let request = QueryBuilder::fuzzy()
    .term("laptop")
    .and("gaming")
    .not("refurbished")
    .limit(20)
    .build_request();

let results = client.search_index::<serde_json::Value>("products", &request).await?;

// Federation search
let results = client.search_federation::<serde_json::Value>("global", &request).await?;
# Ok(())
# }
```

The `QueryBuilder` supports fuzzy, exact, and dynamic query modes, field queries,
range queries, comparisons, boolean composition (`and`/`or`/`not`), grouping,
pagination (`limit`/`offset`), and sorting (`order_by`). `limit` is capped at 200
and is checked client-side before the request is sent.

### AI search summaries

Engine 0.10.0+ can stream an LLM-generated summary of a query's results over
Server-Sent Events. Check [`get_index_capabilities`] first — the endpoint
requires AI features to be enabled on the index and a key with summary
permissions:

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
use searchcraft::search::query::QueryBuilder;
use searchcraft::search::types::SummaryStreamEvent;
use searchcraft::search::StreamExt;

let capabilities = client.get_index_capabilities("products").await?;
if capabilities.ai.enabled {
    let request = QueryBuilder::fuzzy().term("laptop").build_request();
    let mut stream = client.search_summary("products", &request).await?;

    while let Some(event) = stream.next().await {
        match event {
            SummaryStreamEvent::Delta(d) => print!("{}", d.content),
            SummaryStreamEvent::Done(d) => println!("\n({} results)", d.results_count),
            SummaryStreamEvent::Error(e) => eprintln!("summary failed: {}", e.message),
            SummaryStreamEvent::Metadata(_) => {}
        }
    }
}
# Ok(())
# }
```

Malformed frames and mid-stream transport failures arrive as `Error` events
rather than ending the stream. The configured timeout bounds only connection
setup for this endpoint, so a slow generation is never cut short.

[`get_index_capabilities`]: https://docs.rs/searchcraft/latest/searchcraft/struct.SearchcraftClient.html#method.get_index_capabilities

### Cancelling requests

Every method returns an ordinary future, and `search_summary` an ordinary
stream, so cancellation needs no extra API. Dropping the future or stream
before it finishes aborts the request and closes its connection. The examples
below use Tokio.

To stop a summary early, drop the stream:

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
# let request = searchcraft::search::query::QueryBuilder::fuzzy().term("x").build_request();
use searchcraft::search::types::SummaryStreamEvent;
use searchcraft::search::StreamExt;

let mut stream = client.search_summary("products", &request).await?;
let mut summary = String::new();

while let Some(event) = stream.next().await {
    if let SummaryStreamEvent::Delta(d) = event {
        summary.push_str(&d.content);
        if summary.len() >= 500 {
            break;
        }
    }
}
// `break` alone leaves the connection open until `stream` goes out of scope.
drop(stream);
# Ok(())
# }
```

To give one call a deadline shorter than the configured timeout, wrap it in
`tokio::time::timeout`. When the deadline passes, the request is dropped and
you get `Elapsed` back. The configured timeout still applies on its own and
reports `Error::Network`, so you can tell the two apart:

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
# let request = searchcraft::search::query::QueryBuilder::fuzzy().term("x").build_request();
use std::time::Duration;

let search = client.search_index::<serde_json::Value>("products", &request);
match tokio::time::timeout(Duration::from_millis(300), search).await {
    Ok(result) => println!("Found {} results", result?.data.count),
    Err(_elapsed) => println!("Search took too long; cancelled"),
}
# Ok(())
# }
```

For search-as-you-type, race each search against the next query with
`tokio::select!`. If a newer query wins, the stale search is dropped, so a slow
response to an old query can't overwrite the results of a new one:

```rust,no_run
# async fn example(
#     client: searchcraft::SearchcraftClient,
#     mut queries: tokio::sync::mpsc::Receiver<String>,
# ) -> Result<(), searchcraft::Error> {
# fn render<T>(_: T) {}
use searchcraft::search::query::QueryBuilder;

// `queries` receives the search box contents on each keystroke.
let mut next = queries.recv().await;
while let Some(text) = next {
    let request = QueryBuilder::fuzzy().term(text).limit(10).build_request();
    tokio::select! {
        result = client.search_index::<serde_json::Value>("products", &request) => {
            render(result?);
            next = queries.recv().await;
        }
        newer = queries.recv() => next = newer,
    }
}
# Ok(())
# }
```

Cancelling stops the client from waiting. The server may still finish work it
has already started.

### Index management

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
use std::collections::HashMap;
use searchcraft::admin::types::{FieldConfig, FieldType, IndexConfig};

// List indices
let indices = client.list_indices().await?;

// Create an index. Request types have `Default` or a `new` constructor, so you
// only spell out the fields you care about.
let config = IndexConfig {
    language: Some("en".into()),
    search_fields: Some(vec!["title".into()]),
    fields: Some(HashMap::from([(
        "title".to_string(),
        FieldConfig {
            stored: Some(true),
            required: Some(true),
            ..FieldConfig::new(FieldType::Text)
        },
    )])),
    ..Default::default()
};
client.create_index("my-index", &config).await?;

// Update sends only the fields you set, leaving the rest untouched
let patch = IndexConfig { auto_commit_delay: Some(5000), ..Default::default() };
client.update_index("my-index", &patch).await?;

// Get index stats
let stats = client.get_index_stats("my-index").await?;
println!("Documents: {}", stats.document_count);
# Ok(())
# }
```

### Document ingestion

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
let doc = serde_json::json!({
    "id": "doc-1",
    "title": "Gaming Laptop",
    "price": 999.99
});

client.insert_document("products", &doc).await?;

// Batch insert
let docs = vec![doc];
client.batch_insert_documents("products", &docs).await?;
# Ok(())
# }
```

### Additional APIs

| Module         | Endpoints                                          |
|----------------|----------------------------------------------------|
| **Federations**| List (all, by organization), get, create, update, delete, indices, stats |
| **Auth keys**  | List (all, by application/organization/federation/index), get, create, update, delete, permission check (self-hosted) |
| **Capabilities**| Per-index AI capability reporting                 |
| **Health**     | Health check                                       |
| **Stopwords**  | Get, get defaults, add, remove, clear              |
| **Synonyms**   | Get, add, remove, clear                            |
| **Transactions**| Commit, rollback index transactions               |
| **Measure**    | Status, send events, dashboard summary/conversion/usage |

This crate targets Searchcraft engine **0.11.0**. Every endpoint the engine
exposes at that version is covered, except `POST /index/:index/hint`, which is
marked work-in-progress server-side.

## Configuration

```rust,no_run
use std::time::Duration;
use searchcraft::Config;

let config = Config::new(
    "https://my-instance.searchcraft.io",
    Some("read-key"),
    Some("ingest-key"),
)?
.with_admin_key("admin-key")
.with_timeout(Duration::from_secs(10))
.with_header("X-Custom", "value");

let client = searchcraft::SearchcraftClient::from_config(config)?;
# Ok::<(), searchcraft::Error>(())
```

## Error handling

All fallible operations return `searchcraft::error::Result<T>`. The `Error`
enum covers configuration mistakes, authentication failures (401/403),
not-found (404), validation errors (400), other API errors, and network
problems:

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), None::<String>)?;
# let request = searchcraft::search::query::QueryBuilder::fuzzy().term("x").build_request();
match client.search_index::<serde_json::Value>("products", &request).await {
    Ok(response) => println!("Found {} results", response.data.count),
    Err(searchcraft::Error::NotFound(msg)) => eprintln!("Index not found: {msg}"),
    Err(searchcraft::Error::Authentication { status, .. }) => eprintln!("Auth failed ({status})"),
    Err(e) if e.is_retryable() => eprintln!("Transient error, retry: {e}"),
    Err(e) => eprintln!("Error: {e}"),
}
# Ok(())
# }
```

## Minimum supported Rust version

The MSRV is **1.75** (Rust 2021 edition).

## Contributing

Please file issues in the
[Searchcraft Issues](https://github.com/searchcraft-inc/searchcraft-issues)
repository.

Maintainers: see [PUBLISHING.md](PUBLISHING.md) for the release process.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full text.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you shall be licensed as above, without any
additional terms or conditions.
