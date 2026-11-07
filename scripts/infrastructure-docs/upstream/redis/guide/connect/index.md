# Connect to Redis

You can connect to Redis in the following ways:

* With the `redis-cli` command line tool
* Use RedisInsight as a graphical user interface
* Via a client library for your programming language
  
## Redis command line interface

The [Redis command line interface](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/connect/cli.md) (also known as `redis-cli`) is a terminal program that sends commands to and reads replies from the Redis server. It has the following two main modes: 

1. An interactive Read Eval Print Loop (REPL) mode where the user types Redis commands and receives replies.
2. A command mode where `redis-cli` is executed with additional arguments, and the reply is printed to the standard output.

## RedisInsight

[RedisInsight](https://redis.io/docs/connect/insight) combines a graphical user interface with Redis CLI to let you work with any Redis deployment. You can visually browse and interact with data, take advantage of diagnostic tools, learn by example, and much more. Best of all, RedisInsight is free.

## Client libraries

It's easy to connect your application to a Redis database. The official client libraries cover the following languages:

* [C#/.NET](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/connect/clients/dotnet.md)
* [Go](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/connect/clients/go.md)
* [Java](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/connect/clients/java/_index.md)
* [Node.js](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/connect/clients/nodejs.md)
* [Python](https://github.com/redis/redis-doc/blob/1044224477fceacc5af56a636b0a7495d19e533c/docs/connect/clients/python.md)

You can find a complete list of all client libraries, including the community-maintained ones, on the [clients page](https://redis.io/resources/clients/).

<hr/>
