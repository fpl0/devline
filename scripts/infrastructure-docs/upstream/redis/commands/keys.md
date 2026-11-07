# KEYS

Returns all key names that match a pattern.

## Syntax

```text
KEYS pattern
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(N) with N being the number of keys in the database, under the assumption that the key names in the database and the given pattern have limited length.

**Arity** 2

**ACL categories** KEYSPACE, DANGEROUS

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "pattern",
    "type": "pattern"
  }
]
```


## Reply schema

```json
{
  "description": "list of keys matching pattern",
  "type": "array",
  "items": {
    "type": "string"
  }
}
```
