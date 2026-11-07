# ARINFO

Returns metadata about an array.

## Syntax

```text
ARINFO key [FULL]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(1), or O(N) with FULL option where N is the number of slices.

**Arity** -2

**ACL categories** ARRAY

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "full",
    "type": "pure-token",
    "token": "FULL",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "count": {
      "type": "integer",
      "description": "Total number of non-empty elements."
    },
    "len": {
      "type": "integer",
      "description": "Logical length (highest index + 1)."
    },
    "next-insert-index": {
      "type": "integer",
      "description": "Index the next ARINSERT would use, or 0 if unset/exhausted."
    },
    "slices": {
      "type": "integer",
      "description": "Number of allocated slices."
    },
    "directory-size": {
      "type": "integer",
      "description": "Directory allocation capacity (flat dir_alloc or superdir sdir_cap)."
    },
    "super-dir-entries": {
      "type": "integer",
      "description": "Number of super-directory entries (0 if not in superdir mode)."
    },
    "slice-size": {
      "type": "integer",
      "description": "Configured slice size."
    },
    "dense-slices": {
      "type": "integer",
      "description": "Number of dense slices (FULL only)."
    },
    "sparse-slices": {
      "type": "integer",
      "description": "Number of sparse slices (FULL only)."
    },
    "avg-dense-size": {
      "type": "number",
      "description": "Average allocation size of dense slices (FULL only)."
    },
    "avg-dense-fill": {
      "type": "number",
      "description": "Average fill rate of dense slices (FULL only)."
    },
    "avg-sparse-size": {
      "type": "number",
      "description": "Average capacity of sparse slices (FULL only)."
    }
  }
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
