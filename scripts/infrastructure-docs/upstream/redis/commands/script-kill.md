# SCRIPT KILL

Terminates a server-side Lua script during execution.

## Syntax

```text
SCRIPT KILL (null)
```


**Available since** 2.6.0

**Group** scripting

**Time complexity** O(1)

**Arity** 2

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, ALLOW_BUSY

## Reply schema

```json
{
  "const": "OK"
}
```
