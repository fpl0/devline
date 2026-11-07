# SELECT

Changes the selected database.

## Syntax

```text
SELECT index
```


**Available since** 1.0.0

**Group** connection

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** LOADING, STALE, FAST

## Arguments

```json
[
  {
    "name": "index",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
