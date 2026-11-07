# COMMAND GETKEYSANDFLAGS

Extracts the key names and access flags for an arbitrary command.

## Syntax

```text
COMMAND GETKEYSANDFLAGS command [arg [arg ...]]
```


**Available since** 7.0.0

**Group** server

**Time complexity** O(N) where N is the number of arguments to the command

**Arity** -3

**ACL categories** CONNECTION

**Command flags** LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "command",
    "type": "string"
  },
  {
    "name": "arg",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "List of keys from the given Redis command and their usage flags.",
  "type": "array",
  "uniqueItems": true,
  "items": {
    "type": "array",
    "minItems": 2,
    "maxItems": 2,
    "items": [
      {
        "description": "Key name",
        "type": "string"
      },
      {
        "description": "Set of key flags",
        "type": "array",
        "minItems": 1,
        "items": {
          "type": "string"
        }
      }
    ]
  }
}
```
