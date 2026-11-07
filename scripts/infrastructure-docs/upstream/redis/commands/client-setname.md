# CLIENT SETNAME

Sets the connection name.

## Syntax

```text
CLIENT SETNAME connection-name
```


**Available since** 2.6.9

**Group** connection

**Time complexity** O(1)

**Arity** 3

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "connection-name",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
