//! Admin example — demonstrates index and document management.
//!
//! # Usage
//!
//! ```sh
//! export SEARCHCRAFT_URL="https://my-instance.searchcraft.io"
//! export SEARCHCRAFT_READ_KEY="sc-read-..."
//! export SEARCHCRAFT_INGEST_KEY="sc-ingest-..."
//! cargo run --example admin
//! ```

use searchcraft::admin::types::IndexConfig;
use searchcraft::SearchcraftClient;

#[tokio::main]
async fn main() -> Result<(), searchcraft::Error> {
    let url = std::env::var("SEARCHCRAFT_URL").expect("set SEARCHCRAFT_URL");
    let read_key = std::env::var("SEARCHCRAFT_READ_KEY").expect("set SEARCHCRAFT_READ_KEY");
    let ingest_key = std::env::var("SEARCHCRAFT_INGEST_KEY").expect("set SEARCHCRAFT_INGEST_KEY");

    let client = SearchcraftClient::new(&url, Some(&read_key), Some(&ingest_key))?;

    // --- List indices ---
    println!("=== Listing indices ===");
    let indices = client.list_indices().await?;
    for name in &indices.index_names {
        println!("  - {name}");
    }

    // --- Create an index ---
    let index_name = "example-products";
    println!("\n=== Creating index \"{index_name}\" ===");
    let config = IndexConfig::default();
    match client.create_index(index_name, &config).await {
        Ok(msg) => println!("  Created: {msg}"),
        Err(e) => println!("  Skipped (may already exist): {e}"),
    }

    // --- Insert documents ---
    println!("\n=== Inserting documents ===");
    let docs = vec![
        serde_json::json!({
            "id": "prod-1",
            "title": "Gaming Laptop",
            "category": "electronics",
            "price": 1299.99
        }),
        serde_json::json!({
            "id": "prod-2",
            "title": "Wireless Mouse",
            "category": "electronics",
            "price": 29.99
        }),
    ];
    client.batch_insert_documents(index_name, &docs).await?;
    println!("  Inserted {} documents", docs.len());

    // --- Get index stats ---
    println!("\n=== Index stats ===");
    let stats = client.get_index_stats(index_name).await?;
    println!("  Document count: {}", stats.document_count);

    // --- Health check ---
    println!("\n=== Health check ===");
    let health = client.health_check().await?;
    println!("  Status: {} — {}", health.status, health.data);

    Ok(())
}
