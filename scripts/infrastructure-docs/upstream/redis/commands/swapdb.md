# SWAPDB

Swaps two Redis databases.

## Syntax

```text
SWAPDB index1 index2
```


**Available since** 4.0.0

**Group** server

**Time complexity** O(N) where N is the count of clients watching or blocking on keys from both databases.

**Arity** 3

**ACL categories** KEYSPACE, DANGEROUS

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "index1",
    "type": "integer"
  },
  {
    "name": "index2",
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
