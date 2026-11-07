# GEOHASH

Returns members from a geospatial index as geohash strings.

## Syntax

```text
GEOHASH key [member [member ...]]
```


**Available since** 3.2.0

**Group** geo

**Time complexity** O(1) for each member requested.

**Arity** -2

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
    "name": "member",
    "type": "string",
    "multiple": true,
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "An array where each element is the Geohash corresponding to each member name passed as argument to the command.",
  "type": "array",
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
