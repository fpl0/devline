# Dart quick start

> Get started with Meilisearch using the Dart SDK in 5 minutes.

This guide walks you through setting up Meilisearch with Dart and Flutter.

## Prerequisites

* Dart 3.0 or higher (or Flutter 3.0+)
* A Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))

## 1. Install the SDK

Add the dependency to your `pubspec.yaml`:

```yaml
dependencies:
  meilisearch: ^0.17.1
```

Then run:

```bash
dart pub get
# or for Flutter
flutter pub get
```

## 2. Connect to Meilisearch

```dart
import 'package:meilisearch/meilisearch.dart';
import 'dart:io';

final client = MeiliSearchClient(
  Platform.environment['MEILISEARCH_URL']!,
  Platform.environment['MEILISEARCH_KEY'],
);
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

```dart
// Add documents
final movies = [
  {'id': 1, 'title': 'The Matrix', 'genres': ['Action', 'Sci-Fi'], 'year': 1999},
  {'id': 2, 'title': 'Inception', 'genres': ['Action', 'Thriller'], 'year': 2010},
  {'id': 3, 'title': 'Interstellar', 'genres': ['Drama', 'Sci-Fi'], 'year': 2014},
];

final index = client.index('movies');
final task = await index.addDocuments(movies);

// Wait for indexing to complete
await client.waitForTask(task.taskUid);
```

## 4. Search

```dart
final result = await index.search('matrix');

for (final hit in result.hits) {
  print('${hit['title']} (${hit['year']})');
}
// The Matrix (1999)
```

## 5. Search with filters

First, configure filterable attributes:

```dart
await index.updateFilterableAttributes(['genres', 'year']);
```

Then search with filters:

```dart
final result = await index.search(
  '',
  SearchQuery(filter: 'genres = "Sci-Fi" AND year > 2000'),
);
```

## Full example

```dart
import 'package:meilisearch/meilisearch.dart';
import 'dart:io';

void main() async {
  // Connect
  final client = MeiliSearchClient(
    Platform.environment['MEILISEARCH_URL']!,
    Platform.environment['MEILISEARCH_KEY'],
  );

  // Add documents
  final movies = [
    {'id': 1, 'title': 'The Matrix', 'year': 1999},
    {'id': 2, 'title': 'Inception', 'year': 2010},
    {'id': 3, 'title': 'Interstellar', 'year': 2014},
  ];

  final index = client.index('movies');
  final task = await index.addDocuments(movies);
  await client.waitForTask(task.taskUid);

  // Search
  final result = await index.search('inter');
  for (final hit in result.hits) {
    print(hit['title']);
  }
}
```

## Flutter example

```dart
import 'package:flutter/material.dart';
import 'package:meilisearch/meilisearch.dart';

class SearchScreen extends StatefulWidget {
  @override
  _SearchScreenState createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  final client = MeiliSearchClient('YOUR_MEILISEARCH_URL', 'YOUR_SEARCH_KEY');
  List<Map<String, dynamic>> results = [];

  Future<void> search(String query) async {
    final result = await client.index('movies').search(query);
    setState(() {
      results = List<Map<String, dynamic>>.from(result.hits);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        TextField(onChanged: search),
        Expanded(
          child: ListView.builder(
            itemCount: results.length,
            itemBuilder: (context, index) => ListTile(
              title: Text(results[index]['title']),
            ),
          ),
        ),
      ],
    );
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

* [meilisearch-dart on GitHub](https://github.com/meilisearch/meilisearch-dart)
* [Package on pub.dev](https://pub.dev/packages/meilisearch)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.