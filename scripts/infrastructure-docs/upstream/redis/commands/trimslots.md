# TRIMSLOTS

Trim the keys that belong to specified slots.

## Syntax

```text
TRIMSLOTS RANGES numranges startslot endslot [startslot endslot ...]
```


**Available since** 8.4.0

**Group** server

**Time complexity** O(N) where N is the total number of keys in all databases

**Arity** -5

**ACL categories** KEYSPACE, DANGEROUS

**Command flags** WRITE

## Arguments

```json
[
  {
    "name": "ranges",
    "token": "RANGES",
    "type": "block",
    "arguments": [
      {
        "name": "numranges",
        "type": "integer"
      },
      {
        "name": "slots",
        "type": "block",
        "multiple": true,
        "arguments": [
          {
            "name": "startslot",
            "type": "integer"
          },
          {
            "name": "endslot",
            "type": "integer"
          }
        ]
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
