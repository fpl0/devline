# GEODIST

Returns the distance between two members of a geospatial index.

## Syntax

```text
GEODIST key member1 member2 [M|KM|FT|MI]
```


**Available since** 3.2.0

**Group** geo

**Time complexity** O(1)

**Arity** -4

**ACL categories** GEO

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
    "name": "member1",
    "type": "string"
  },
  {
    "name": "member2",
    "type": "string"
  },
  {
    "name": "unit",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "m",
        "type": "pure-token",
        "token": "m"
      },
      {
        "name": "km",
        "type": "pure-token",
        "token": "km"
      },
      {
        "name": "ft",
        "type": "pure-token",
        "token": "ft"
      },
      {
        "name": "mi",
        "type": "pure-token",
        "token": "mi"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "one or both of elements are missing",
      "type": "null"
    },
    {
      "description": "distance as a double (represented as a string) in the specified units",
      "type": "string",
      "pattern": "^[0-9]*(.[0-9]*)?$"
    }
  ]
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
