# MEMORY STATS

Returns details about memory usage.

## Syntax

```text
MEMORY STATS (null)
```


**Available since** 4.0.0

**Group** server

**Time complexity** O(1)

**Arity** 2

## Reply schema

```json
{
  "description": "memory usage details",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "peak.allocated": {
      "type": "integer"
    },
    "total.allocated": {
      "type": "integer"
    },
    "startup.allocated": {
      "type": "integer"
    },
    "replication.backlog": {
      "type": "integer"
    },
    "replica.fullsync.buffer": {
      "type": "integer"
    },
    "clients.slaves": {
      "type": "integer"
    },
    "clients.normal": {
      "type": "integer"
    },
    "clients.normal.shared": {
      "type": "integer"
    },
    "clients.normal.unshared": {
      "type": "integer"
    },
    "cluster.links": {
      "type": "integer"
    },
    "aof.buffer": {
      "type": "integer"
    },
    "lua.caches": {
      "type": "integer"
    },
    "script.VMs": {
      "type": "integer"
    },
    "functions.caches": {
      "type": "integer"
    },
    "overhead.db.hashtable.lut": {
      "type": "integer"
    },
    "overhead.db.hashtable.rehashing": {
      "type": "integer"
    },
    "overhead.total": {
      "type": "integer"
    },
    "hash.templates": {
      "type": "integer"
    },
    "db.dict.rehashing.count": {
      "type": "integer"
    },
    "keys.count": {
      "type": "integer"
    },
    "keys.bytes-per-key": {
      "type": "integer"
    },
    "dataset.bytes": {
      "type": "integer"
    },
    "dataset.percentage": {
      "type": "number"
    },
    "peak.percentage": {
      "type": "number"
    },
    "allocator.allocated": {
      "type": "integer"
    },
    "allocator.active": {
      "type": "integer"
    },
    "allocator.resident": {
      "type": "integer"
    },
    "allocator.muzzy": {
      "type": "integer"
    },
    "allocator-fragmentation.ratio": {
      "type": "number"
    },
    "allocator-fragmentation.bytes": {
      "type": "integer"
    },
    "allocator-rss.ratio": {
      "type": "number"
    },
    "allocator-rss.bytes": {
      "type": "integer"
    },
    "rss-overhead.ratio": {
      "type": "number"
    },
    "rss-overhead.bytes": {
      "type": "integer"
    },
    "fragmentation": {
      "type": "number"
    },
    "fragmentation.bytes": {
      "type": "integer"
    }
  },
  "patternProperties": {
    "^db\\.\\d+$": {
      "type": "object",
      "properties": {
        "overhead.hashtable.main": {
          "type": "integer"
        },
        "overhead.hashtable.expires": {
          "type": "integer"
        }
      },
      "additionalProperties": false
    }
  }
}
```
