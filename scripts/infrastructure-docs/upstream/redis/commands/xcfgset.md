# XCFGSET

Sets the IDMP configuration parameters for a stream.

## Syntax

```text
XCFGSET key [IDMP-DURATION idmp-duration] [IDMP-MAXSIZE idmp-maxsize]
```


**Available since** 8.6.0

**Group** stream

**Time complexity** O(1)

**Arity** -2

**ACL categories** STREAM

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "idmp-duration",
    "type": "integer",
    "token": "IDMP-DURATION",
    "optional": true
  },
  {
    "name": "idmp-maxsize",
    "type": "integer",
    "token": "IDMP-MAXSIZE",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "UPDATE"
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
