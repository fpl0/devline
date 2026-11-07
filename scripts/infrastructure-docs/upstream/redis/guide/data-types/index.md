# Understand Redis data types

Redis is a data structure server.
At its core, Redis provides a collection of native data types that help you solve a wide variety of problems, from [caching](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/manual/client-side-caching.md) to [queuing](https://redis.io/docs/data-types/lists/) to [event processing](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/streams.md).
Below is a short description of each data type, with links to broader overviews and command references.

If you'd like to try a comprehensive tutorial for each data structure, see their overview pages below.


## Core

### Strings 

[Redis strings](https://redis.io/docs/data-types/strings) are the most basic Redis data type, representing a sequence of bytes.
For more information, see:

* [Overview of Redis strings](https://redis.io/docs/data-types/strings/)
* [Redis string command reference](https://redis.io/commands/?group=string)

### Lists

[Redis lists](https://redis.io/docs/data-types/lists) are lists of strings sorted by insertion order.
For more information, see:

* [Overview of Redis lists](https://redis.io/docs/data-types/lists/)
* [Redis list command reference](https://redis.io/commands/?group=list)

### Sets

[Redis sets](https://redis.io/docs/data-types/sets) are unordered collections of unique strings that act like the sets from your favorite programming language (for example, [Java HashSets](https://docs.oracle.com/javase/7/docs/api/java/util/HashSet.html), [Python sets](https://docs.python.org/3.10/library/stdtypes.html#set-types-set-frozenset), and so on).
With a Redis set, you can add, remove, and test for existence in O(1) time (in other words, regardless of the number of set elements).
For more information, see:

* [Overview of Redis sets](https://redis.io/docs/data-types/sets/)
* [Redis set command reference](https://redis.io/commands/?group=set)

### Hashes

[Redis hashes](https://redis.io/docs/data-types/hashes) are record types modeled as collections of field-value pairs.
As such, Redis hashes resemble [Python dictionaries](https://docs.python.org/3/tutorial/datastructures.html#dictionaries), [Java HashMaps](https://docs.oracle.com/javase/8/docs/api/java/util/HashMap.html), and [Ruby hashes](https://ruby-doc.org/core-3.1.2/Hash.html).
For more information, see:

* [Overview of Redis hashes](https://redis.io/docs/data-types/hashes/)
* [Redis hashes command reference](https://redis.io/commands/?group=hash)

### Sorted sets

[Redis sorted sets](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/sorted-sets.md) are collections of unique strings that maintain order by each string's associated score.
For more information, see:

* [Overview of Redis sorted sets](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/sorted-sets.md)
* [Redis sorted set command reference](https://redis.io/commands/?group=sorted-set)

### Streams

A [Redis stream](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/streams.md) is a data structure that acts like an append-only log.
Streams help record events in the order they occur and then syndicate them for processing.
For more information, see:

* [Overview of Redis Streams](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/streams.md)
* [Redis Streams command reference](https://redis.io/commands/?group=stream)

### Geospatial indexes

[Redis geospatial indexes](https://redis.io/docs/data-types/geospatial) are useful for finding locations within a given geographic radius or bounding box.
For more information, see:

* [Overview of Redis geospatial indexes](https://redis.io/docs/data-types/geospatial/)
* [Redis geospatial indexes command reference](https://redis.io/commands/?group=geo)

### Bitmaps

[Redis bitmaps](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/bitmaps.md) let you perform bitwise operations on strings. 
For more information, see:

* [Overview of Redis bitmaps](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/bitmaps.md)
* [Redis bitmap command reference](https://redis.io/commands/?group=bitmap)

### Bitfields

[Redis bitfields](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/bitfields.md) efficiently encode multiple counters in a string value.
Bitfields provide atomic get, set, and increment operations and support different overflow policies.
For more information, see:

* [Overview of Redis bitfields](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/bitfields.md)
* The `BITFIELD` command.

### HyperLogLog

The [Redis HyperLogLog](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/probabilistic/hyperloglogs.md) data structures provide probabilistic estimates of the cardinality (i.e., number of elements) of large sets. For more information, see:

* [Overview of Redis HyperLogLog](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/data-types/probabilistic/hyperloglogs.md)
* [Redis HyperLogLog command reference](https://redis.io/commands/?group=hyperloglog)

## Extensions

To extend the features provided by the included data types, use one of these options:

1. Write your own custom [server-side functions in Lua](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/interact/programmability/_index.md).
1. Write your own Redis module using the [modules API](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/reference/modules/_index.md) or check out the [community-supported modules](https://redis.io/docs/modules/).
1. Use [JSON](https://redis.io/docs/stack/json/), [querying](https://redis.io/docs/stack/search/), [time series](https://redis.io/docs/stack/timeseries/), and other capabilities provided by [Redis Stack](https://redis.io/docs/stack/).

<hr>
