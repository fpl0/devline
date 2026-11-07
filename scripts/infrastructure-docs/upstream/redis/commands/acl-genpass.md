# ACL GENPASS

Generates a pseudorandom, secure password that can be used to identify ACL users.

## Syntax

```text
ACL GENPASS [bits]
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(1)

**Arity** -2

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "bits",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "type": "string",
  "description": "Pseudorandom data. By default it contains 64 bytes, representing 256 bits of data. If `bits` was given, the output string length is the number of specified bits (rounded to the next multiple of 4) divided by 4."
}
```
