# Go quick start

> Get started with Meilisearch using the Go SDK in 5 minutes.

This guide walks you through setting up Meilisearch with Go.

## Prerequisites

* Go 1.16 or higher
* A Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))

## 1. Install the SDK

```bash
go get github.com/meilisearch/meilisearch-go
```

## 2. Connect to Meilisearch

```go
package main

import (
    "os"
    "github.com/meilisearch/meilisearch-go"
)

func main() {
    client := meilisearch.New(
        os.Getenv("MEILISEARCH_URL"),
        meilisearch.WithAPIKey(os.Getenv("MEILISEARCH_KEY")),
    )
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

```go
type Movie struct {
    ID     int      `json:"id"`
    Title  string   `json:"title"`
    Genres []string `json:"genres"`
    Year   int      `json:"year"`
}

movies := []Movie{
    {ID: 1, Title: "The Matrix", Genres: []string{"Action", "Sci-Fi"}, Year: 1999},
    {ID: 2, Title: "Inception", Genres: []string{"Action", "Thriller"}, Year: 2010},
    {ID: 3, Title: "Interstellar", Genres: []string{"Drama", "Sci-Fi"}, Year: 2014},
}

// Add documents to the 'movies' index
task, _ := client.Index("movies").AddDocuments(movies)

// Wait for indexing to complete
client.WaitForTask(task.TaskUID)
```

## 4. Search

```go
results, _ := client.Index("movies").Search("matrix", nil)

fmt.Println(results.Hits)
// [map[id:1 title:The Matrix genres:[Action Sci-Fi] year:1999]]
```

## 5. Search with filters

First, configure filterable attributes:

```go
client.Index("movies").UpdateFilterableAttributes(&[]string{"genres", "year"})
```

Then search with filters:

```go
results, _ := client.Index("movies").Search("", &meilisearch.SearchRequest{
    Filter: "genres = 'Sci-Fi' AND year > 2000",
})
```

## Full example

```go
package main

import (
    "fmt"
    "os"
    "github.com/meilisearch/meilisearch-go"
)

type Movie struct {
    ID     int      `json:"id"`
    Title  string   `json:"title"`
    Genres []string `json:"genres"`
    Year   int      `json:"year"`
}

func main() {
    client := meilisearch.New(
        os.Getenv("MEILISEARCH_URL"),
        meilisearch.WithAPIKey(os.Getenv("MEILISEARCH_KEY")),
    )

    // Add documents
    movies := []Movie{
        {ID: 1, Title: "The Matrix", Genres: []string{"Action", "Sci-Fi"}, Year: 1999},
        {ID: 2, Title: "Inception", Genres: []string{"Action", "Thriller"}, Year: 2010},
        {ID: 3, Title: "Interstellar", Genres: []string{"Drama", "Sci-Fi"}, Year: 2014},
    }

    task, _ := client.Index("movies").AddDocuments(movies)
    client.WaitForTask(task.TaskUID)

    // Search
    results, _ := client.Index("movies").Search("inter", nil)
    fmt.Println(results.Hits)
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

* [meilisearch-go on GitHub](https://github.com/meilisearch/meilisearch-go)
* [SDK documentation](https://pkg.go.dev/github.com/meilisearch/meilisearch-go)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.