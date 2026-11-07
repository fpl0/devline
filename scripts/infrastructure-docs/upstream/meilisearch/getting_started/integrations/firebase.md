# Firebase

> Sync your Firestore documents to Meilisearch using the official Firebase extension.

The official [firestore-meilisearch](https://github.com/meilisearch/firestore-meilisearch) extension automatically synchronizes documents from a Cloud Firestore collection to a Meilisearch index, enabling full-text search on your Firestore data.

> [!NOTE]
>
> [Meilisearch Cloud](https://www.meilisearch.com/cloud?utm_campaign=oss\&utm_source=docs\&utm_medium=firebase) is the easiest way to get a Meilisearch instance for use with Firebase.

## Prerequisites

* A Firebase project on the Blaze (pay-as-you-go) plan
* Cloud Firestore set up in your Firebase project
* A running Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))
* A Meilisearch API key with write permissions

## Install the extension

You can install the extension using the Firebase Console or the Firebase CLI.

> [tab] Firebase Console
>
> Visit the [Firebase Extensions Hub](https://extensions.dev/extensions/meilisearch/firestore-meilisearch) and click **Install**.
>
> Follow the prompts to select your Firebase project and configure the extension.

> [tab] Firebase CLI
>
> ```bash theme={null}
> firebase ext:install meilisearch/firestore-meilisearch --project=YOUR_PROJECT_ID
> ```
>
> Replace `YOUR_PROJECT_ID` with your Firebase project ID.

## Configuration

During installation, you'll configure the following parameters:

| Parameter | Description |
| - | - |
| **Cloud Functions location** | Region where the extension's functions will be deployed |
| **Collection path** | The Firestore collection to sync (e.g., `products`, `articles`) |
| **Fields to index** | Comma-separated list of field names, or leave blank to index all fields |
| **Meilisearch index name** | The name of the Meilisearch index to sync data to |
| **Meilisearch host** | Your Meilisearch instance URL (must start with `http://` or `https://`) |
| **Meilisearch API key** | An API key with permission to manage indexes |

## How it works

Once installed, the extension deploys a Cloud Function called `indexingWorker` that:

1. **Listens** for document creates, updates, and deletions in your specified collection
2. **Syncs** changes to your Meilisearch index in real-time
3. **Maps** Firestore document IDs to a `_firestore_id` field in Meilisearch

> [!WARNING]
>
> The extension only monitors the specified collection, not subcollections. Install additional instances to sync other collections.

## Data format

### Document IDs

Firestore document IDs are automatically mapped to a `_firestore_id` field in Meilisearch. Any field named `_firestore_id` in your source documents will be ignored.

### Geolocation

For geo search functionality, name your GeoPoint field `_geo` in Firestore. Meilisearch will automatically recognize it for [geo search queries](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started).

```javascript
// Firestore document with geo data
{
  name: "Eiffel Tower",
  _geo: new firebase.firestore.GeoPoint(48.8584, 2.2945)
}
```

## Import existing documents

The extension only syncs documents created or modified after installation. To import existing documents, use the provided import script:

```bash
npx @meilisearch/firestore-meilisearch-scripts import
```

> [!TIP]
>
> See the [import script documentation](https://github.com/meilisearch/firestore-meilisearch/blob/main/guides/IMPORT_EXISTING_DOCUMENTS.md) for detailed instructions.

## Search your data

Once your data is synced, you can search it using any Meilisearch SDK or the REST API:

> [tab] JavaScript
>
> ```javascript theme={null}
> import { Meilisearch } from 'meilisearch'
>
> const client = new Meilisearch({
>   host: process.env.MEILISEARCH_URL,
>   apiKey: process.env.MEILISEARCH_KEY
> })
>
> const results = await client.index('products').search('phone')
> console.log(results.hits)
> ```

> [tab] cURL
>
> ```bash theme={null}
> curl "${MEILISEARCH_URL}/indexes/products/search" \
>   -H "Authorization: Bearer ${MEILISEARCH_KEY}" \
>   -H "Content-Type: application/json" \
>   -d '{"q": "phone"}'
> ```

## Resources

* [GitHub repository](https://github.com/meilisearch/firestore-meilisearch)
* [Firebase Extensions Hub](https://extensions.dev/extensions/meilisearch/firestore-meilisearch)
* [Meilisearch blog: Firebase + Meilisearch](https://www.meilisearch.com/blog/firebase-meilisearch)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.