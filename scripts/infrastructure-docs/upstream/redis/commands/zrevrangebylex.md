# ZREVRANGEBYLEX

Returns members in a sorted set within a lexicographical range in reverse order.

## Syntax

```text
ZREVRANGEBYLEX key max min [LIMIT offset count]
```


**Available since** 2.8.9

**Group** sorted_set

**Time complexity** O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements being returned. If M is constant (e.g. always asking for the first 10 elements with LIMIT), you can consider it O(log(N)).

**Arity** -4

**Deprecated since** 6.2.0

**Replacement** `ZRANGE` with the `REV` and `BYLEX` arguments

**ACL categories** SORTEDSET

**Command flags** READONLY

**Documentation flags** DEPRECATED

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "max",
    "type": "string"
  },
  {
    "name": "min",
    "type": "string"
  },
  {
    "token": "LIMIT",
    "name": "limit",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "offset",
        "type": "integer"
      },
      {
        "name": "count",
        "type": "integer"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "List of the elements in the specified score range.",
  "uniqueItems": true,
  "items": {
    "type": "string"
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
