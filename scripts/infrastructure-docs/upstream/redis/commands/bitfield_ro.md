# BITFIELD_RO

Performs arbitrary read-only bitfield integer operations on strings.

## Syntax

```text
BITFIELD_RO key [GET encoding offset [GET encoding offset ...]]
```


**Available since** 6.0.0

**Group** bitmap

**Time complexity** O(1) for each subcommand specified

**Arity** -2

**ACL categories** BITMAP

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "token": "GET",
    "name": "get-block",
    "type": "block",
    "optional": true,
    "multiple": true,
    "multiple_token": true,
    "arguments": [
      {
        "name": "encoding",
        "type": "string"
      },
      {
        "name": "offset",
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
  "items": {
    "description": "The result of the subcommand at the same position",
    "type": "integer"
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
