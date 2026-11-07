# Rust quick start

> Get started with Meilisearch using the Rust SDK in 5 minutes.

This guide walks you through setting up Meilisearch with Rust.

## Prerequisites

* Rust stable (1.65+)
* A Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))

## 1. Install the SDK

Add to your `Cargo.toml`:

```toml
[dependencies]
meilisearch-sdk = "0.27"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

## 2. Connect to Meilisearch

```rust
use meilisearch_sdk::client::Client;
use std::env;

#[tokio::main]
async fn main() {
    let client = Client::new(
        env::var("MEILISEARCH_URL").unwrap(),
        Some(env::var("MEILISEARCH_KEY").unwrap())
    ).unwrap();
}
```

> [!NOTE]
>
> **Set your environment variables:**
>
> ```bash theme={null}
> export MEILISEARCH_URL="https://your-instance.meilisearch.io"  # or http://localhost:7700
> export MEILISEARCH_KEY="your_api_key"
> ```
>
> [Get a free Cloud instance →](https://cloud.meilisearch.com)

## 3. Add documents

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Movie {
    id: i32,
    title: String,
    genres: Vec<String>,
    year: i32,
}

let movies = vec![
    Movie { id: 1, title: "The Matrix".to_string(), genres: vec!["Action".to_string(), "Sci-Fi".to_string()], year: 1999 },
    Movie { id: 2, title: "Inception".to_string(), genres: vec!["Action".to_string(), "Thriller".to_string()], year: 2010 },
    Movie { id: 3, title: "Interstellar".to_string(), genres: vec!["Drama".to_string(), "Sci-Fi".to_string()], year: 2014 },
];

// Add documents to the 'movies' index
let task = client.index("movies").add_documents(&movies, Some("id")).await.unwrap();

// Wait for indexing to complete
task.wait_for_completion(&client, None, None).await.unwrap();
```

## 4. Search

```rust
let results: SearchResults<Movie> = client
    .index("movies")
    .search()
    .with_query("matrix")
    .execute()
    .await
    .unwrap();

println!("{:?}", results.hits);
```

## 5. Search with filters

First, configure filterable attributes:

```rust
client.index("movies")
    .set_filterable_attributes(&["genres", "year"])
    .await
    .unwrap();
```

Then search with filters:

```rust
let results: SearchResults<Movie> = client
    .index("movies")
    .search()
    .with_filter(r#"genres = "Sci-Fi" AND year > 2000"#)
    .execute()
    .await
    .unwrap();
```

## Full example

```rust
use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::SearchResults;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct Movie {
    id: i32,
    title: String,
    genres: Vec<String>,
    year: i32,
}

#[tokio::main]
async fn main() {
    let client = Client::new(
        env::var("MEILISEARCH_URL").unwrap(),
        Some(env::var("MEILISEARCH_KEY").unwrap())
    ).unwrap();

    // Add documents
    let movies = vec![
        Movie { id: 1, title: "The Matrix".to_string(), genres: vec!["Action".to_string(), "Sci-Fi".to_string()], year: 1999 },
        Movie { id: 2, title: "Inception".to_string(), genres: vec!["Action".to_string(), "Thriller".to_string()], year: 2010 },
        Movie { id: 3, title: "Interstellar".to_string(), genres: vec!["Drama".to_string(), "Sci-Fi".to_string()], year: 2014 },
    ];

    let task = client.index("movies").add_documents(&movies, Some("id")).await.unwrap();
    task.wait_for_completion(&client, None, None).await.unwrap();

    // Search
    let results: SearchResults<Movie> = client
        .index("movies")
        .search()
        .with_query("inter")
        .execute()
        .await
        .unwrap();

    println!("{:?}", results.hits);
}
```

## Next steps

> [card] [Full-text search](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy)
>
> Configure ranking and relevancy

> [card] [Filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started)
>
> Add filters and facets

> [card] [AI-powered search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started)
>
> Add semantic search

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post)
>
> Explore all search parameters

## Resources

* [meilisearch-rust on GitHub](https://github.com/meilisearch/meilisearch-rust)
* [SDK documentation](https://docs.rs/meilisearch-sdk)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.