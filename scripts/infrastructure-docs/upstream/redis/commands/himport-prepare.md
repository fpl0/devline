# HIMPORT PREPARE

Defines a session-local fieldset that maps a name to a sorted set of field names.

## Syntax

```text
HIMPORT PREPARE fieldset-name field [field ...]
```


**Available since** 8.10.0

**Group** hash

**Time complexity** O(N*log(N)) where N is the number of fields.

**Arity** -4

**ACL categories** HASH

**Command flags** DENYOOM

## Arguments

```json
[
  {
    "name": "fieldset-name",
    "type": "string"
  },
  {
    "name": "field",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
