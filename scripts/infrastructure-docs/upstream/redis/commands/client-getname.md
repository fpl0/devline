# CLIENT GETNAME

Returns the name of the connection.

## Syntax

```text
CLIENT GETNAME (null)
```


**Available since** 2.6.9

**Group** connection

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "oneOf": [
    {
      "type": "string",
      "description": "The connection name of the current connection"
    },
    {
      "type": "null",
      "description": "Connection name was not set"
    }
  ]
}
```
