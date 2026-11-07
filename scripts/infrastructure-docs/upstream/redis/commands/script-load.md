# SCRIPT LOAD

Loads a server-side Lua script to the script cache.

## Syntax

```text
SCRIPT LOAD script
```


**Available since** 2.6.0

**Group** scripting

**Time complexity** O(N) with N being the length in bytes of the script body.

**Arity** 3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, STALE

## Arguments

```json
[
  {
    "name": "script",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The SHA1 digest of the script added into the script cache",
  "type": "string"
}
```
