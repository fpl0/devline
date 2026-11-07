# RANDOMKEY

Returns a random key name from the database.

## Syntax

```text
RANDOMKEY (null)
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 1

**ACL categories** KEYSPACE

**Command flags** READONLY, TOUCHES_ARBITRARY_KEYS

## Reply schema

```json
{
  "oneOf": [
    {
      "description": "when the database is empty",
      "type": "null"
    },
    {
      "description": "random key in db",
      "type": "string"
    }
  ]
}
```
