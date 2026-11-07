# ACL DRYRUN

Simulates the execution of a command by a user, without executing the command.

## Syntax

```text
ACL DRYRUN username command [arg [arg ...]]
```


**Available since** 7.0.0

**Group** server

**Time complexity** O(1).

**Arity** -4

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "username",
    "type": "string"
  },
  {
    "name": "command",
    "type": "string"
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
  "anyOf": [
    {
      "const": "OK",
      "description": "The given user may successfully execute the given command."
    },
    {
      "type": "string",
      "description": "The description of the problem, in case the user is not allowed to run the given command."
    }
  ]
}
```
