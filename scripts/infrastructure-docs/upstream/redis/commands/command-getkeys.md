# COMMAND GETKEYS

Extracts the key names from an arbitrary command.

## Syntax

```text
COMMAND GETKEYS command [arg [arg ...]]
```


**Available since** 2.8.13

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
  "description": "List of keys from the given Redis command.",
  "type": "array",
  "items": {
    "type": "string"
  },
  "uniqueItems": true
}
```
