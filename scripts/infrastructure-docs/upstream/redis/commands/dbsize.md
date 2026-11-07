# DBSIZE

Returns the number of keys in the database.

## Syntax

```text
DBSIZE (null)
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(1)

**Arity** 1

**ACL categories** KEYSPACE

**Command flags** READONLY, FAST

## Reply schema

```json
{
  "type": "integer",
  "description": "The number of keys in the currently-selected database."
}
```
