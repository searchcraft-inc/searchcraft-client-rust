# searchcraft

[![Crates.io](https://img.shields.io/crates/v/searchcraft.svg)](https://crates.io/crates/searchcraft)
[![docs.rs](https://docs.rs/searchcraft/badge.svg)](https://docs.rs/searchcraft)
[![License](https://img.shields.io/crates/l/searchcraft.svg)](LICENSE)

An async Rust client for the [Searchcraft](https://searchcraft.io) search API.

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
        println!("{} (score: {:.2})", hit.document_id, hit.score);
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
pagination (`limit`/`offset`), and sorting (`order_by`).

### Index management

```rust,no_run
# async fn example() -> Result<(), searchcraft::Error> {
# let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
use searchcraft::admin::types::IndexConfig;

// List indices
let indices = client.list_indices().await?;

// Create an index
let config = IndexConfig::default();
client.create_index("my-index", &config).await?;

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
| **Federations**| List, get, stats                                   |
| **Auth keys**  | List, get, create, update, delete (self-hosted)    |
| **Health**     | Health check                                       |
| **Stopwords**  | Get, set per-index stopword lists                  |
| **Synonyms**   | Get, set per-index synonym maps                    |
| **Transactions**| Commit, rollback index transactions               |
| **Measure**    | Send analytics/measure events                      |

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

## Publishing to crates.io

> **Note:** These are manual steps for the crate maintainer. End users
> consume the crate via `cargo add searchcraft` once it is published.

### Prerequisites

1. Create an account on [crates.io](https://crates.io) and generate an API
   token under **Account Settings → API Tokens**.
2. Log in locally:
   ```sh
   cargo login <your-token>
   ```

### Pre-publish checklist

```sh
# Format check
cargo fmt --all --check

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Dry-run publish (validates packaging without uploading)
cargo publish --dry-run
```

### Publish

```sh
cargo publish
```

After publishing, the crate will be available at
[crates.io/crates/searchcraft](https://crates.io/crates/searchcraft) and
users can add it with:

```sh
cargo add searchcraft
```

### Releasing a new version

1. Update the `version` field in `Cargo.toml`.
2. Run the pre-publish checklist above.
3. Commit and tag: `git tag v<version> && git push --tags`
4. Run `cargo publish`.

## Minimum supported Rust version

The MSRV is **1.75** (Rust 2021 edition).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
