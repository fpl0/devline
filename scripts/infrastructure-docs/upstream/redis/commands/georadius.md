# GEORADIUS

Queries a geospatial index for members within a distance from a coordinate, optionally stores the result.

## Syntax

```text
GEORADIUS key longitude latitude radius M|KM|FT|MI [WITHCOORD] [WITHDIST] [WITHHASH] [COUNT count [ANY]] [ASC|DESC] [STORE key|STOREDIST key]
```


**Available since** 3.2.0

**Group** geo

**Time complexity** O(N+log(M)) where N is the number of elements inside the bounding box of the circular area delimited by center and radius and M is the number of items inside the index.

**Arity** -6

**Deprecated since** 6.2.0

**Replacement** `GEOSEARCH` and `GEOSEARCHSTORE` with the `BYRADIUS` argument

**ACL categories** GEO

**Command flags** WRITE, DENYOOM

**Documentation flags** DEPRECATED

## History

- 6.2.0 — Added the `ANY` option for `COUNT`.
- 7.0.0 — Added support for uppercase unit names.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "longitude",
    "type": "double"
  },
  {
    "name": "latitude",
    "type": "double"
  },
  {
    "name": "radius",
    "type": "double"
  },
  {
    "name": "unit",
    "type": "oneof",
    "arguments": [
      {
        "name": "m",
        "type": "pure-token",
        "token": "m"
      },
      {
        "name": "km",
        "type": "pure-token",
        "token": "km"
      },
      {
        "name": "ft",
        "type": "pure-token",
        "token": "ft"
      },
      {
        "name": "mi",
        "type": "pure-token",
        "token": "mi"
      }
    ]
  },
  {
    "name": "withcoord",
    "token": "WITHCOORD",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "withdist",
    "token": "WITHDIST",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "withhash",
    "token": "WITHHASH",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "count-block",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "token": "COUNT",
        "name": "count",
        "type": "integer"
      },
      {
        "name": "any",
        "token": "ANY",
        "type": "pure-token",
        "optional": true,
        "since": "6.2.0"
      }
    ]
  },
  {
    "name": "order",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "asc",
        "type": "pure-token",
        "token": "ASC"
      },
      {
        "name": "desc",
        "type": "pure-token",
        "token": "DESC"
      }
    ]
  },
  {
    "name": "store",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "token": "STORE",
        "name": "storekey",
        "display": "key",
        "type": "key",
        "key_spec_index": 1
      },
      {
        "token": "STOREDIST",
        "name": "storedistkey",
        "display": "key",
        "type": "key",
        "key_spec_index": 2
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "Array of matched members information",
  "anyOf": [
    {
      "description": "If no WITH* option is specified, array of matched members names",
      "type": "array",
      "items": {
        "description": "name",
        "type": "string"
      }
    },
    {
      "type": "array",
      "items": {
        "type": "array",
        "minItems": 1,
        "maxItems": 4,
        "items": [
          {
            "description": "Matched member name",
            "type": "string"
          }
        ],
        "additionalItems": {
          "oneOf": [
            {
              "description": "If WITHDIST option is specified, the distance from the center as a floating point number, in the same unit specified in the radius",
              "type": "string"
            },
            {
              "description": "If WITHHASH option is specified, the geohash integer",
              "type": "integer"
            },
            {
              "description": "If WITHCOORD option is specified, the coordinates as a two items x,y array (longitude,latitude)",
              "type": "array",
              "minItems": 2,
              "maxItems": 2,
              "items": [
                {
                  "description": "latitude (x)",
                  "type": "number"
                },
                {
                  "description": "longitude (y)",
                  "type": "number"
                }
              ]
            }
          ]
        }
      }
    },
    {
      "description": "number of items stored in key",
      "type": "integer"
    }
  ]
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
        "pos": 1
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
    "notes": "Incomplete because duplicate STORE options use last-wins; fall back to georadiusGetKeys",
    "flags": [
      "OW",
      "UPDATE",
      "INCOMPLETE"
    ],
    "begin_search": {
      "keyword": {
        "keyword": "STORE",
        "startfrom": 6
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
    "notes": "Incomplete because duplicate STOREDIST options use last-wins; fall back to georadiusGetKeys",
    "flags": [
      "OW",
      "UPDATE",
      "INCOMPLETE"
    ],
    "begin_search": {
      "keyword": {
        "keyword": "STOREDIST",
        "startfrom": 6
      }
    },
    "find_keys": {
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
