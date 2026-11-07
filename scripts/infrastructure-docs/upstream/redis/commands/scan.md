# SCAN

Iterates over the key names in the database.

## Syntax

```text
SCAN cursor [MATCH pattern] [COUNT count] [TYPE type]
```


**Available since** 2.8.0

**Group** generic

**Time complexity** O(1) for every call. O(N) for a complete iteration, including enough command calls for the cursor to return back to 0. N is the number of elements inside the collection.

**Arity** -2

**ACL categories** KEYSPACE

**Command flags** READONLY, TOUCHES_ARBITRARY_KEYS

## History

- 6.0.0 — Added the `TYPE` subcommand.

## Arguments

```json
[
  {
    "name": "cursor",
    "type": "integer"
  },
  {
    "token": "MATCH",
    "name": "pattern",
    "type": "pattern",
    "optional": true
  },
  {
    "token": "COUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  },
  {
    "token": "TYPE",
    "name": "type",
    "type": "string",
    "optional": true,
    "since": "6.0.0"
  }
]
```


## Reply schema

```json
{
  "description": "cursor and scan response in array form",
  "type": "array",
  "minItems": 2,
  "maxItems": 2,
  "items": [
    {
      "description": "cursor",
      "type": "string"
    },
    {
      "description": "list of keys",
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  ]
}
```
