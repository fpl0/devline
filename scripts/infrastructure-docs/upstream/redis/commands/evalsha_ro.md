# EVALSHA_RO

Executes a read-only server-side Lua script by SHA1 digest.

## Syntax

```text
EVALSHA_RO sha1 numkeys [key [key ...]] [arg [arg ...]]
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** Depends on the script that is executed.

**Arity** -3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, SKIP_MONITOR, NO_MANDATORY_KEYS, STALE, SCRIPT_RUNNER, READONLY

## Arguments

```json
[
  {
    "name": "sha1",
    "type": "string"
  },
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "optional": true,
    "multiple": true
  },
  {
    "name": "arg",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "Return value depends on the script that is executed"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 2
      }
    },
    "find_keys": {
      "keynum": {
        "keynumidx": 0,
        "firstkey": 1,
        "step": 1
      }
    }
  }
]
```
