# Ruby quick start

> Get started with Meilisearch using the Ruby SDK in 5 minutes.

This guide walks you through setting up Meilisearch with Ruby.

## Prerequisites

* Ruby 2.7 or higher
* A Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))

## 1. Install the SDK

```bash
gem install meilisearch
```

Or add to your Gemfile:

```ruby
gem 'meilisearch'
```

## 2. Connect to Meilisearch

```ruby
require 'meilisearch'

client = MeiliSearch::Client.new(
  ENV['MEILISEARCH_URL'],
  ENV['MEILISEARCH_KEY']
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

```ruby
movies = [
  { id: 1, title: 'The Matrix', genres: ['Action', 'Sci-Fi'], year: 1999 },
  { id: 2, title: 'Inception', genres: ['Action', 'Thriller'], year: 2010 },
  { id: 3, title: 'Interstellar', genres: ['Drama', 'Sci-Fi'], year: 2014 }
]

# Add documents to the 'movies' index
task = client.index('movies').add_documents(movies)

# Wait for indexing to complete
client.wait_for_task(task['taskUid'])
```

## 4. Search

```ruby
results = client.index('movies').search('matrix')

puts results['hits']
# [{"id"=>1, "title"=>"The Matrix", "genres"=>["Action", "Sci-Fi"], "year"=>1999}]
```

## 5. Search with filters

First, configure filterable attributes:

```ruby
client.index('movies').update_filterable_attributes(['genres', 'year'])
```

Then search with filters:

```ruby
results = client.index('movies').search('', {
  filter: 'genres = "Sci-Fi" AND year > 2000'
})
```

## Full example

```ruby
require 'meilisearch'

client = MeiliSearch::Client.new(
  ENV['MEILISEARCH_URL'],
  ENV['MEILISEARCH_KEY']
)

# Add documents
movies = [
  { id: 1, title: 'The Matrix', genres: ['Action', 'Sci-Fi'], year: 1999 },
  { id: 2, title: 'Inception', genres: ['Action', 'Thriller'], year: 2010 },
  { id: 3, title: 'Interstellar', genres: ['Drama', 'Sci-Fi'], year: 2014 }
]

task = client.index('movies').add_documents(movies)
client.wait_for_task(task['taskUid'])

# Search
results = client.index('movies').search('inter')
puts results['hits']
```

## Rails integration

For Rails applications, use the meilisearch-rails gem:

```bash
gem install meilisearch-rails
```

Add to your model:

```ruby
class Movie < ApplicationRecord
  include MeiliSearch::Rails

  meilisearch do
    attribute :title, :genres, :year
    searchable_attributes [:title]
    filterable_attributes [:genres, :year]
  end
end
```

[See the full Rails guide →](https://www.meilisearch.com/docs/getting_started/frameworks/rails)

## Next steps

> [card] [Rails integration](https://www.meilisearch.com/docs/getting_started/frameworks/rails)
>
> Full Rails integration guide

> [card] [Full-text search](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy)
>
> Configure ranking and relevancy

> [card] [Filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started)
>
> Add filters and facets

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post)
>
> Explore all search parameters

## Resources

* [meilisearch-ruby on GitHub](https://github.com/meilisearch/meilisearch-ruby)
* [meilisearch-rails on GitHub](https://github.com/meilisearch/meilisearch-rails)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.