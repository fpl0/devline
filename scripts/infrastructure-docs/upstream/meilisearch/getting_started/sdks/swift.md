# Swift quick start

> Get started with Meilisearch using the Swift SDK in 5 minutes.

This guide walks you through setting up Meilisearch with Swift for iOS, macOS, and server-side Swift applications.

## Prerequisites

* Swift 5.5 or higher
* Xcode 13+ (for iOS/macOS development)
* A Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))

## 1. Install the SDK

> [tab] Swift Package Manager
>
> Add the dependency to your `Package.swift`:
>
> ```swift theme={null}
> dependencies: [
>     .package(url: "https://github.com/meilisearch/meilisearch-swift.git", from: "0.17.0")
> ]
> ```
>
> Or in Xcode: File → Add Packages → Enter the repository URL.

> [tab] CocoaPods
>
> Add to your `Podfile`:
>
> ```ruby theme={null}
> pod 'MeiliSearch'
> ```
>
> Then run `pod install`.

## 2. Connect to Meilisearch

```swift
import MeiliSearch

let client = try! MeiliSearch(
    host: ProcessInfo.processInfo.environment["MEILISEARCH_URL"]!,
    apiKey: ProcessInfo.processInfo.environment["MEILISEARCH_KEY"]
)
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

```swift
// Define your document struct
struct Movie: Codable, Equatable {
    let id: Int
    let title: String
    let genres: [String]
    let year: Int
}

// Add documents
let movies = [
    Movie(id: 1, title: "The Matrix", genres: ["Action", "Sci-Fi"], year: 1999),
    Movie(id: 2, title: "Inception", genres: ["Action", "Thriller"], year: 2010),
    Movie(id: 3, title: "Interstellar", genres: ["Drama", "Sci-Fi"], year: 2014)
]

let index = client.index("movies")

// Using async/await
let task = try await index.addDocuments(documents: movies)
try await client.waitForTask(taskUid: task.taskUid)
```

## 4. Search

```swift
let searchResult: Searchable<Movie> = try await index.search("matrix")

for hit in searchResult.hits {
    print("\(hit.title) (\(hit.year))")
}
// The Matrix (1999)
```

## 5. Search with filters

First, configure filterable attributes:

```swift
try await index.updateFilterableAttributes(["genres", "year"])
```

Then search with filters:

```swift
let searchParams = SearchParameters(
    query: "",
    filter: "genres = \"Sci-Fi\" AND year > 2000"
)

let results: Searchable<Movie> = try await index.search(searchParams)
```

## Full example

```swift
import MeiliSearch

// Connect
let client = try! MeiliSearch(
    host: ProcessInfo.processInfo.environment["MEILISEARCH_URL"]!,
    apiKey: ProcessInfo.processInfo.environment["MEILISEARCH_KEY"]
)

struct Movie: Codable, Equatable {
    let id: Int
    let title: String
    let year: Int
}

Task {
    // Add documents
    let movies = [
        Movie(id: 1, title: "The Matrix", year: 1999),
        Movie(id: 2, title: "Inception", year: 2010),
        Movie(id: 3, title: "Interstellar", year: 2014)
    ]

    let index = client.index("movies")
    let task = try await index.addDocuments(documents: movies)
    try await client.waitForTask(taskUid: task.taskUid)

    // Search
    let results: Searchable<Movie> = try await index.search("inter")
    for hit in results.hits {
        print(hit.title)
    }
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

* [meilisearch-swift on GitHub](https://github.com/meilisearch/meilisearch-swift)
* [SDK documentation](https://github.com/meilisearch/meilisearch-swift#readme)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.