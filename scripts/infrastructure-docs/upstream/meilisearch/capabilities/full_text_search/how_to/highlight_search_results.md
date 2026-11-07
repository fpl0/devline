# Highlight search results

> Highlight and crop matched terms in search results to help users quickly see why a document was returned.

Highlighting wraps matched query terms in HTML tags so your frontend can visually emphasize them. Cropping trims long text fields to show only the relevant portion around matched terms. Both features work through search parameters and return their results in the `_formatted` object of each hit.

## Highlight specific attributes

Use `attributesToHighlight` to specify which fields should have matched terms wrapped in tags:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "knight",
    "attributesToHighlight": ["title", "overview"]
  }'
```

Matched terms appear in the `_formatted` object wrapped in `<em>` tags:

```json
{
  "_formatted": {
    "title": "The Dark <em>Knight</em>",
    "overview": "When the menace known as the Joker wreaks havoc, the Dark <em>Knight</em> must..."
  }
}
```

## Highlight all attributes

Set `attributesToHighlight` to `["*"]` to highlight matched terms across all [displayed attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes):

> [!NOTE]
>
> `attributesToHighlight` highlights matches within any attribute you list, even attributes that are not part of `searchableAttributes`. Meilisearch does not produce new matches in non-searchable fields, but if the query term happens to appear verbatim in one of those fields, it will still be wrapped in highlight tags.

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "knight",
    "attributesToHighlight": ["*"]
  }'
```

## Custom highlight tags

Replace the default `<em>` tags with any markup using `highlightPreTag` and `highlightPostTag`:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "knight",
    "attributesToHighlight": ["title"],
    "highlightPreTag": "<strong class=\"highlight\">",
    "highlightPostTag": "</strong>"
  }'
```

Result:

```json
{
  "_formatted": {
    "title": "The Dark <strong class=\"highlight\">Knight</strong>"
  }
}
```

## Crop long text fields

Use `attributesToCrop` to trim long fields and show only the portion around the matched terms:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "battle",
    "attributesToCrop": ["overview"],
    "cropLength": 20
  }'
```

Result:

```json
{
  "_formatted": {
    "overview": "...the epic battle between good and evil reaches its climax as..."
  }
}
```

### Crop parameters reference

| Parameter | Type | Default | Description |
| - | - | - | - |
| `attributesToCrop` | Array of strings | `null` | Attributes to crop. Use `["*"]` for all displayed attributes. |
| `cropLength` | Integer | `10` | Maximum number of words in the cropped result. |
| `cropMarker` | String | `"..."` | String inserted at the beginning or end of cropped text. |

> [!NOTE]
>
> `cropLength` counts every word in the returned snippet, including query terms and stop words. For example, if `cropLength` is `2` and you search for `shifu`, the cropped value might look like `"…Shifu informs…"`, containing two words in total (the query term plus one surrounding word).

> [!NOTE]
>
> Crop markers are only inserted where content has been removed. If the cropped snippet begins at the first word of the field, no marker is added to the start; likewise, no marker is added to the end when the snippet reaches the end of the field.

### Per-attribute crop length

Override the global `cropLength` for specific attributes by appending `:length` to the attribute name:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "mystery",
    "attributesToCrop": ["overview:40", "tagline:10"]
  }'
```

### Custom crop marker

Replace the default `"..."` truncation marker:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "mystery",
    "attributesToCrop": ["overview"],
    "cropMarker": " [...]"
  }'
```

## Combine highlighting and cropping

For the best user experience, use both features together to show short, relevant snippets with visually emphasized matches:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/movies/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "space adventure",
    "attributesToHighlight": ["title", "overview"],
    "attributesToCrop": ["overview"],
    "cropLength": 25,
    "highlightPreTag": "<mark>",
    "highlightPostTag": "</mark>",
    "cropMarker": "..."
  }'
```

Result:

```json
{
  "_formatted": {
    "title": "<mark>Space</mark> Odyssey",
    "overview": "...embark on a daring <mark>space</mark> <mark>adventure</mark> to save humanity from..."
  }
}
```

> [!TIP]
>
> Attributes listed in `attributesToCrop` are automatically included in the `_formatted` response. If the same attribute appears in both `attributesToCrop` and `attributesToHighlight`, the cropped text will also have matched terms highlighted.

> [!NOTE]
>
> For the full parameter reference, see the [search API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post).


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.