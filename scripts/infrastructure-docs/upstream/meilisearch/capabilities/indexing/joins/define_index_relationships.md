# Define index relationships

> Create relationships between documents across indices using joins, reducing data duplication while maintaining flexibility for filtering and hydration.

Joins enable you to define relationships between documents in different indices, similar to foreign keys in relational databases. Instead of duplicating data across documents, you store only IDs and use joins to hydrate full documents at search time or filter by related data.

## Why use joins?

> [card] Reduce data duplication
>
> Store company details once; reference from many deals instead of embedding full objects in each deal document.

> [card] Faster indexing
>
> Smaller documents mean faster indexing operations and reduced storage requirements.

> [card] Keep data normalized
>
> Update company information once and it's automatically reflected in all deal queries.

> [card] Flexible queries
>
> Hydrate related data when needed and filter by related document properties without denormalization.

## How relationships work

A relationship connects a **source index** to a **target index** using foreign key fields. For example, a document in the `deals` index references a document in the `companies` index through a `company_id` field:

```json
{
  "id": "deal_1",
  "title": "Contract X",
  "company_id": "company_1"
}
```

```json
{
  "id": "company_1",
  "name": "Acme Inc"
}
```

Once the relationship is configured, Meilisearch automatically hydrates the related document in search results, replacing `company_id` with the full company object.

## Define a relationship

Configure foreign keys and filterable attributes in the source index settings:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/deals/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "company_id",
        "foreignIndexUid": "companies"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["company_id"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

### Configuration parameters

| Parameter | Type | Description | Example |
| - | - | - | - |
| `fieldName` | string | Field in the source index containing the ID(s) | `"company_id"` or `"actor_ids"` |
| `foreignIndexUid` | string | UID of the target index | `"companies"` or `"actors"` |
| `attributePatterns` | array | Field patterns to make filterable | `["company_id"]` |
| `features` | object | Filter capabilities (equality, comparison, facetSearch) | `{"filter": {"equality": true}}` |

## Relationship types

### One-to-one

Each source document has exactly one related target document.

**Example:** Users → Profiles (each user has one profile)

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/users/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "profile_id",
        "foreignIndexUid": "profiles"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["profile_id"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

### One-to-many

Each source document has multiple related target documents, stored as an array of IDs.

**Example:** Companies → Employees (one company has many employees)

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/companies/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "employee_ids",
        "foreignIndexUid": "employees"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["employee_ids"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

### Many-to-many

Multiple source documents reference multiple target documents, typically using array fields.

**Example:** Films → Actors (one film has many actors, many films feature the same actor)

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/films/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "actor_ids",
        "foreignIndexUid": "actors"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["actor_ids"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

### Self-references

You can create relationships where a document references other documents in the same index. This is useful for modeling hierarchical or interconnected data.

**Example:** Products frequently bought together

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/products/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "frequently_bought_with",
        "foreignIndexUid": "products"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["frequently_bought_with"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

### Multiple relationships

Configure multiple foreign keys in a single settings update:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/deals/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "company_id",
        "foreignIndexUid": "companies"
      },
      {
        "fieldName": "owner_user_id",
        "foreignIndexUid": "users"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["company_id"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      },
      {
        "attributePatterns": ["owner_user_id"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

## Update relationships

To replace an existing relationship, provide the updated configuration:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/deals/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": [
      {
        "fieldName": "assigned_user_id",
        "foreignIndexUid": "users"
      }
    ],
    "filterableAttributes": [
      {
        "attributePatterns": ["assigned_user_id"],
        "features": {
          "facetSearch": false,
          "filter": {
            "equality": true,
            "comparison": false
          }
        }
      }
    ]
  }'
```

To remove all relationships:

```bash
curl \
  -X PATCH 'MEILISEARCH_URL/indexes/deals/settings' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "foreignKeys": []
  }'
```

## Depth limitation

Joins support hydration and filtering at one level only. You cannot create nested chains like deals → companies → industry\_details. Each hydration or filter operates on a direct relationship between two indices.

## Data integrity

Meilisearch does not enforce referential integrity. You can create foreign key references to non-existent documents, and deleting a target document does not affect documents that reference it.

When a referenced document is deleted:

* **Hydration** returns the document UID instead of the full object
* **Filtering** ignores the reference in filter comparisons

### Cleanup strategy

You can handle orphaned references in two ways: delete the source documents entirely, or remove the orphaned IDs from the foreign key fields.

#### Delete source documents

Use the [delete by filter](https://www.meilisearch.com/docs/reference/api/documents/delete-documents-by-filter) API to remove source documents that reference non-existent targets:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/deals/delete-by-filter' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "filter": "company_id = deleted_company_1 OR company_id = deleted_company_2"
  }'
```

#### Remove orphaned IDs with functions

Use [edit documents by function](https://www.meilisearch.com/docs/capabilities/indexing/how_to/edit_documents_with_functions) to remove specific orphaned IDs from array foreign key fields without deleting the source documents. This is useful for many-to-many relationships where only some referenced IDs are orphaned.

For example, remove a deleted actor from the `actors` array in all film documents:

```bash
curl \
  -X POST 'MEILISEARCH_URL/indexes/films/documents/edit' \
  -H 'Authorization: Bearer API_KEY' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "function": "doc.actors = doc.actors.filter(|id| id != context.deleted_id)",
    "context": {
      "deleted_id": "actor_42"
    },
    "filter": "actors = actor_42"
  }'
```

## Next steps

> [card] [Foreign filters](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filtering_by_joined_data)
>
> Filter documents by properties of related data across indices

> [card] [Precise array filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/precise_filtering_array_items)
>
> Use AND logic to filter array relationships with joins

> [card] [RBAC with joins](https://www.meilisearch.com/docs/capabilities/security/advanced/rbac_with_joins)
>
> Implement role-based access control using joins and tenant tokens

> [card] [Indexing best practices](https://www.meilisearch.com/docs/capabilities/indexing/advanced/indexing_best_practices)
>
> Learn best practices including how joins reduce index size


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.