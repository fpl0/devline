# HIMPORT DISCARD

Removes a single session-local fieldset by name.

## Syntax

```text
HIMPORT DISCARD fieldset-name
```


**Available since** 8.10.0

**Group** hash

**Time complexity** O(N) where N is the number of session-local fieldsets.

**Arity** 3

**ACL categories** HASH

## Arguments

```json
[
  {
    "name": "fieldset-name",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "1 if the fieldset was removed, 0 if it did not exist."
}
```
