# SUBSTR

Returns a substring from a string value.

## Syntax

```text
SUBSTR key start end
```


**Available since** 1.0.0

**Group** string

**Time complexity** O(N) where N is the length of the returned string. The complexity is ultimately determined by the returned length, but because creating a substring from an existing string is very cheap, it can be considered O(1) for small strings.

**Arity** 4

**Deprecated since** 2.0.0

**Replacement** `GETRANGE`

**ACL categories** STRING

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
    "name": "start",
    "type": "integer"
  },
  {
    "name": "end",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "type": "string",
  "description": "The substring of the string value stored at key, determined by the offsets start and end (both are inclusive)."
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
