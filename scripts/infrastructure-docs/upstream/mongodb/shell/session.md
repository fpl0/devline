# Session and transaction methods

The Session Class. Represents a server session

- `getDatabase` — Returns a database class that will pass the session to the server with every command

- `advanceOperationTime` — Updates the operation time

- `advanceClusterTime` — Advances the clusterTime for a Session to the provided clusterTime.

- `endSession` — Ends the session

- `hasEnded` — Returns a boolean that specifies whether the session has ended.

- `getClusterTime` — Returns the most recent cluster time as seen by the session. Applicable for replica sets and sharded clusters only.

- `getOperationTime` — Returns the timestamp of the last acknowledged operation for the session.

- `getOptions` — Returns the options object passed to startSession

- `startTransaction` — Starts a multi-document transaction for the session.

- `commitTransaction` — Commits the session’s transaction.

- `abortTransaction` — Aborts the session’s transaction.

- `withTransaction` — Run a function within a transaction context.

- `bulkWrite` — Performs multiple write operations across databases and collections with controls for order of execution, using this session.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Session/)

## `session.withTransaction(() => { session.getDatabase("test").test.insertOne({ doc: 1 }); })`

Run a function within a transaction context.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Session.abortTransaction/#Session.withTransaction)

## `session.bulkWrite(operations, options)`

Performs multiple write operations across databases and collections with controls for order of execution, using this session.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.bulkWrite)
