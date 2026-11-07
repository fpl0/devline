# Image search with multimodal embeddings

> This article shows you the main steps for performing multimodal text-to-image searches

This guide shows the main steps to search through a database of images using Meilisearch's experimental multimodal embeddings.

## Enable multimodal embeddings

First, enable the `multimodal` experimental feature:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/experimental-features/' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "multimodal": true
  }'
```

You may also enable multimodal in your Meilisearch Cloud project's general settings, under "Experimental features".

## Configure a multimodal embedder

Much like other embedders, multimodal embedders must set their `source` to `rest` and explicitly declare their `url`. Depending on your chosen provider, you may also have to specify `apiKey`.

All multimodal embedders must contain an `indexingFragments` field and a `searchFragments` field. Fragments are sets of embeddings built out of specific parts of document data.

Fragments must follow the structure defined by the REST API of your chosen provider.

### `indexingFragments`

Use `indexingFragments` to tell Meilisearch how to send document data to the provider's API when generating document embeddings.

For example, when using VoyageAI's multimodal model, an indexing fragment might look like this:

```json
"indexingFragments": {
  "TEXTUAL_FRAGMENT_NAME": {
    "value": {
      "content": [
        {
          "type": "text",
          "text": "A document named {{doc.title}} described as {{doc.description}}"
        }
      ]
    }
  },
  "IMAGE_FRAGMENT_NAME": {
    "value": {
      "content": [
        {
          "type": "image_url",
          "image_url": "{{doc.poster_url}}"
        }
      ]
    }
  }
}
```

The example above requests Meilisearch to create two sets of embeddings during indexing: one for the textual description of an image, and another for the actual image.

Any JSON string value appearing in a fragment is handled as a Liquid template, where you interpolate document data present in `doc`. In `IMAGE_FRAGMENT_NAME`, that's  `image_url` which outputs the plain URL string in the document field `poster_url`. In `TEXT_FRAGMENT_NAME`, `text` contains a longer string contextualizing two document fields, `title` and `description`.

### `searchFragments`

Use `searchFragments` to tell Meilisearch how to send search query data to the chosen provider's REST API when converting them into embeddings:

```json
"searchFragments": {
  "USER_TEXT_FRAGMENT": {
    "value": {
      "content": [
        {
          "type": "text",
          "text": "{{q}}"
        }
      ]
    }
  },
  "USER_SUBMITTED_IMAGE_FRAGMENT": {
    "value": {
      "content": [
        {
          "type": "image_base64",
          "image_base64": "data:{{media.image.mime}};base64,{{media.image.data}}"
        }
      ]
    }
  }
}
```

In this example, two modes of search are configured:

1. A textual search based on the `q` parameter, which will be embedded as text
2. An image search based on [data url](https://developer.mozilla.org/en-US/docs/Web/URI/Reference/Schemes/data) rebuilt from the `image.mime` and `image.data` field in the `media` field of the query

Search fragments have access to data present in the query parameters `media` and `q`.

Each semantic search query for this embedder should match exactly one search fragment of this embedder, so the fragments should each have at least one disambiguating field.

> [!WARNING]
>
> `media` must match a single search fragment. If `media` matches more than one fragment, or matches no search fragments at all, Meilisearch returns an error.
>
> When you use `media`:
>
> * it is mandatory to specify an embedder (via `hybrid.embedder`).
> * `media` is incompatible with `vector`. Pass one or the other, never both.

### Complete embedder configuration

Your embedder should look similar to this example with all fragments and embedding provider data:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/INDEX_NAME/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "embedders": {
      "MULTIMODAL_EMBEDDER_NAME": {
        "source": "rest",
        "url": "https://api.voyageai.com/v1/multimodal-embeddings",
        "apiKey": "VOYAGE_API_KEY",
        "indexingFragments": {
          "TEXTUAL_FRAGMENT_NAME": {
            "value": {
              "content": [
                {
                  "type": "text",
                  "text": "A document named {{doc.title}} described as {{doc.description}}"
                }
              ]
            }
          },
          "IMAGE_FRAGMENT_NAME": {
            "value": {
              "content": [
                {
                  "type": "image_url",
                  "image_url": "{{doc.poster_url}}"
                }
              ]
            }
          }
        },
        "searchFragments": {
          "USER_TEXT_FRAGMENT": {
            "value": {
              "content": [
                {
                  "type": "text",
                  "text": "{{q}}"
                }
              ]
            }
          },
          "USER_SUBMITTED_IMAGE_FRAGMENT": {
            "value": {
              "content": [
                {
                  "type": "image_base64",
                  "image_base64": "data:{{media.image.mime}};base64,{{media.image.data}}"
                }
              ]
            }
          }
        },
        "request": {
          "inputs": ["{{fragment}}", "{{..}}"],
          "model": "voyage-multimodal-3"
        },
        "response": {
          "data": [
            { "embedding": "{{embedding}}" },
            "{{..}}"
          ]
        }
      }
    }
  }'
```

Since the `source` of this embedder is `rest`, you must also specify a `request` and a `response` fields. These respectively instruct Meilisearch on how to structure the request sent to the embeddings provider, and where to find the embeddings in the provider's response.

> [!TIP]
>
> For a complete, provider-specific walkthrough that also covers video clips and combined text and image queries, see [Multimodal search with TwelveLabs Marengo](https://www.meilisearch.com/docs/capabilities/hybrid_search/providers/twelvelabs).

## Add documents

Once your embedder is configured, you can [add documents to your index](https://www.meilisearch.com/docs/getting_started/first_project) with the [`/documents` endpoint](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get).

During indexing, Meilisearch will automatically generate multimodal embeddings for each document using the configured `indexingFragments`.

## Perform searches

The final step is to perform searches using different types of content.

### Use text to search for images

Use the following search query to retrieve a mix of documents with images matching the description and documents containing the specified keywords:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "a mountain sunset with snow",
    "hybrid": {
      "embedder": "MULTIMODAL_EMBEDDER_NAME"
    }
  }'
```

### Use an image to search for images

You can also use an image to search for other, similar images:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/INDEX_NAME/search' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "media": {
      "image": {
        "mime": "image/jpeg",
        "data": "<BASE64_ENCODED_IMAGE>"
      }
    },
    "hybrid": {
      "embedder": "MULTIMODAL_EMBEDDER_NAME"
    }
  }'
```

> [!TIP]
>
> Image-to-image search requires converting the user's image to Base64 format before sending it to Meilisearch. Your application must handle this conversion client-side.

### Convert images to Base64 on the client

To search with a user-submitted image, read it as a data URL and extract the MIME type and Base64 data:

```javascript
async function imageToSearchPayload(file) {
  return new Promise((resolve) => {
    const reader = new FileReader();
    reader.onload = () => {
      // reader.result is "data:<mime>;base64,<data>"
      const [header, data] = reader.result.split(',');
      const mime = header.match(/data:(.*);base64/)[1];
      resolve({ mime, data });
    };
    reader.readAsDataURL(file);
  });
}

// Usage with a file input
const file = document.getElementById('image-input').files[0];
const { mime, data } = await imageToSearchPayload(file);

const response = await fetch(
  `${MEILISEARCH_URL}/indexes/images/search`,
  {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${API_KEY}`
    },
    body: JSON.stringify({
      hybrid: {
        semanticRatio: 1.0,
        embedder: 'multimodal'
      },
      media: {
        image: { mime, data }
      }
    })
  }
);
```

The `media.image.mime` and `media.image.data` fields in the search request correspond to the `{{media.image.mime}}` and `{{media.image.data}}` template variables used in the `searchFragments` configuration above.

> [!NOTE]
>
> Large images increase request payload size and embedding latency. Consider resizing images to a maximum of 1024x1024 pixels before encoding. The embedding provider handles any further resizing internally.

## Conclusion

With multimodal embedders you can:

1. Configure Meilisearch to embed both images and queries
2. Add image documents. Meilisearch automatically generates embeddings
3. Accept text or image input from users
4. Run hybrid searches using a mix of textual and non-textual input, or run pure semantic searches using only non-textual input


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.