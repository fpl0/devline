# XREAD

Returns messages from multiple streams with IDs greater than the ones requested. Blocks until a message is available otherwise.

## Syntax

```text
XREAD [COUNT count] [MAXCOUNT maxcount] [MAXSIZE maxsize] [BLOCK milliseconds] STREAMS key [key ...] id [id ...]
```


**Available since** 5.0.0

**Group** stream

**Arity** -4

**ACL categories** STREAM

**Command flags** BLOCKING, READONLY

## History

- 8.10.0 — Added the `MAXCOUNT` and `MAXSIZE` options.

## Arguments

```json
[
  {
    "token": "COUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  },
  {
    "token": "MAXCOUNT",
    "name": "maxcount",
    "type": "integer",
    "optional": true,
    "since": "8.10.0"
  },
  {
    "token": "MAXSIZE",
    "name": "maxsize",
    "type": "integer",
    "optional": true,
    "since": "8.10.0"
  },
  {
    "token": "BLOCK",
    "name": "milliseconds",
    "type": "integer",
    "optional": true
  },
  {
    "name": "streams",
    "token": "STREAMS",
    "type": "block",
    "arguments": [
      {
        "name": "key",
        "type": "key",
        "key_spec_index": 0,
        "multiple": true
      },
      {
        "name": "ID",
        "type": "string",
        "multiple": true
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
      "description": "A map of key-value elements when each element composed of key name and the entries reported for that key",
      "type": "object",
      "patternProperties": {
        "^.*$": {
          "description": "The entries reported for that key",
          "type": "array",
          "items": {
            "type": "array",
            "minItems": 2,
            "maxItems": 2,
            "items": [
              {
                "description": "entry id",
                "type": "string",
                "pattern": "[0-9]+-[0-9]+"
              },
              {
                "description": "array of field-value pairs",
                "type": "array",
                "items": {
                  "type": "string"
                }
              }
            ]
          }
        }
      }
    },
    {
      "description": "If BLOCK option is given, and a timeout occurs, or there is no stream we can serve",
      "type": "null"
    }
  ]
}
```


## Key specifications

```json
[
  {
    "notes": "Incomplete because a stream key named STREAMS (or options before it) can shift the STREAMS keyword; fall back to xreadGetKeys",
    "flags": [
      "RO",
      "ACCESS",
      "INCOMPLETE"
    ],
    "begin_search": {
      "keyword": {
        "keyword": "STREAMS",
        "startfrom": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": -1,
        "step": 1,
        "limit": 2
      }
    }
  }
]
```
