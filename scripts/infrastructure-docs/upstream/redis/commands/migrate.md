# MIGRATE

Atomically transfers a key from one Redis instance to another.

## Syntax

```text
MIGRATE host port key|"" destination-db timeout [COPY] [REPLACE] [AUTH password|AUTH2 username password] [KEYS key [key ...]]
```


**Available since** 2.6.0

**Group** generic

**Time complexity** This command actually executes a DUMP+DEL in the source instance, and a RESTORE in the target instance. See the pages of these commands for time complexity. Also an O(N) data transfer between the two instances is performed.

**Arity** -6

**ACL categories** KEYSPACE, DANGEROUS

**Command flags** WRITE

## History

- 3.0.0 — Added the `COPY` and `REPLACE` options.
- 3.0.6 — Added the `KEYS` option.
- 4.0.7 — Added the `AUTH` option.
- 6.0.0 — Added the `AUTH2` option.

## Arguments

```json
[
  {
    "name": "host",
    "type": "string"
  },
  {
    "name": "port",
    "type": "integer"
  },
  {
    "name": "key-selector",
    "type": "oneof",
    "arguments": [
      {
        "name": "key",
        "type": "key",
        "key_spec_index": 0
      },
      {
        "name": "empty-string",
        "type": "pure-token",
        "token": "\"\""
      }
    ]
  },
  {
    "name": "destination-db",
    "type": "integer"
  },
  {
    "name": "timeout",
    "type": "integer"
  },
  {
    "name": "copy",
    "token": "COPY",
    "type": "pure-token",
    "optional": true,
    "since": "3.0.0"
  },
  {
    "name": "replace",
    "token": "REPLACE",
    "type": "pure-token",
    "optional": true,
    "since": "3.0.0"
  },
  {
    "name": "authentication",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "token": "AUTH",
        "name": "auth",
        "display": "password",
        "type": "string",
        "since": "4.0.7"
      },
      {
        "token": "AUTH2",
        "name": "auth2",
        "type": "block",
        "since": "6.0.0",
        "arguments": [
          {
            "name": "username",
            "type": "string"
          },
          {
            "name": "password",
            "type": "string"
          }
        ]
      }
    ]
  },
  {
    "token": "KEYS",
    "name": "keys",
    "display": "key",
    "type": "key",
    "key_spec_index": 1,
    "optional": true,
    "multiple": true,
    "since": "3.0.6"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "const": "OK",
      "description": "Success."
    },
    {
      "const": "NOKEY",
      "description": "No keys were found in the source instance."
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS",
      "DELETE"
    ],
    "begin_search": {
      "index": {
        "pos": 3
      }
    },
    "find_keys": {
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  },
  {
    "flags": [
      "RW",
      "ACCESS",
      "DELETE",
      "INCOMPLETE"
    ],
    "begin_search": {
      "keyword": {
        "keyword": "KEYS",
        "startfrom": -2
      }
    },
    "find_keys": {
      "range": {
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
