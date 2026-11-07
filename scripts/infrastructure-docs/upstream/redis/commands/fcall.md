# FCALL

Invokes a function.

## Syntax

```text
FCALL function numkeys [key [key ...]] [arg [arg ...]]
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** Depends on the function that is executed.

**Arity** -3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, SKIP_MONITOR, MAY_REPLICATE, NO_MANDATORY_KEYS, SCRIPT_RUNNER, STALE

## Arguments

```json
[
  {
    "name": "function",
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
  "description": "Return value depends on the function that is executed"
}
```


## Key specifications

```json
[
  {
    "notes": "We cannot tell how the keys will be used so we assume the worst, RW and UPDATE",
    "flags": [
      "RW",
      "ACCESS",
      "UPDATE"
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
