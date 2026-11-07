# COMMAND LIST

Returns a list of command names.

## Syntax

```text
COMMAND LIST [FILTERBY MODULE module-name|ACLCAT category|PATTERN pattern]
```


**Available since** 7.0.0

**Group** server

**Time complexity** O(N) where N is the total number of Redis commands

**Arity** -2

**ACL categories** CONNECTION

**Command flags** LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "filterby",
    "token": "FILTERBY",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "module-name",
        "type": "string",
        "token": "MODULE"
      },
      {
        "name": "category",
        "type": "string",
        "token": "ACLCAT"
      },
      {
        "name": "pattern",
        "type": "pattern",
        "token": "PATTERN"
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
    "description": "command name",
    "type": "string"
  },
  "uniqueItems": true
}
```
