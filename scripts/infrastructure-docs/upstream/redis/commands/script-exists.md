# SCRIPT EXISTS

Determines whether server-side Lua scripts exist in the script cache.

## Syntax

```text
SCRIPT EXISTS sha1 [sha1 ...]
```


**Available since** 2.6.0

**Group** scripting

**Time complexity** O(N) with N being the number of scripts to check (so checking a single script is an O(1) operation).

**Arity** -3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT

## Arguments

```json
[
  {
    "name": "sha1",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "An array of integers that correspond to the specified SHA1 digest arguments.",
  "type": "array",
  "items": {
    "oneOf": [
      {
        "description": "sha1 hash exists in script cache",
        "const": 1
      },
      {
        "description": "sha1 hash does not exist in script cache",
        "const": 0
      }
    ]
  }
}
```
