# Connection methods

The Mongo Class. Represents a connection to a server

- `connect` — Creates a connection to a MongoDB instance and returns the reference to the database.

- `getDB` — Returns the specified Database of the Mongo object.

- `getCollection` — Returns the specified Collection of the Mongo object.

- `getURI` — Returns the connection string for current session

- `getDBs` — Returns information about all databases. Uses the listDatabases command.

- `bulkWrite` — Performs multiple write operations across databases and collections with controls for order of execution.

- `getDBNames` — Returns an array of all database names. Uses the listDatabases command.

- `close` — Closes a Mongo object, disposing of related resources and closing the underlying connection.

- `getReadPrefMode` — Returns the ReadPreference Mode set for the connection.

- `getReadPrefTagSet` — Returns the ReadPreference TagSet set for the connection.

- `getReadPref` — Returns the ReadPreference set for the connection.

- `getReadConcern` — Returns the ReadConcern set for the connection.

- `getWriteConcern` — Returns the WriteConcern set for the connection.

- `setReadPref` — Sets the ReadPreference for the connection

- `setReadConcern` — Sets the ReadConcern for the connection

- `setWriteConcern` — Sets the WriteConcern for the connection

- `startSession` — Starts a session for the connection.

- `setCausalConsistency` — This method is deprecated. It is not possible to set causal consistency for an entire connection due to driver limitations, use startSession({causalConsistency: <>}) instead.

- `isCausalConsistency` — This method is deprecated. Causal consistency for drivers is set via Mongo.startSession and can be checked via session.getOptions. The default value is true

- `setSlaveOk` — This method is deprecated

- `setSecondaryOk` — This method is deprecated. Use .setReadPref() instead

- `watch` — Opens a change stream cursor on the connection

- `getClientEncryption` — Returns the ClientEncryption object for the current database collection. The ClientEncryption object supports explicit (manual) encryption and decryption of field values for Client-Side field level encryption.

- `getKeyVault` — Returns the KeyVault object for the current database connection. The KeyVault object supports data encryption key management for Client-side field level encryption.

- `convertShardKeyToHashed` — Returns the hashed value for the input using the same hashing function as a hashed index.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo/#Mongo)

## `const collection = db.getMongo().getCollection("databaseName.collectionName")`

Returns the specified Collection of the Mongo object.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getCollection)

## `db.getMongo().bulkWrite(operations, options)`

Performs multiple write operations across databases and collections with controls for order of execution.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.bulkWrite)

## `db.getMongo().getDBNames().map(name => db.getSiblingDB(name).getCollectionNames())`

Returns an array of all database names. Uses the listDatabases command.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getDBs)

## `db.getMongo().getReadPrefMode()`

Returns the ReadPreference Mode set for the connection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getReadPrefMode)

## `db.getMongo().getReadPrefTagSet()`

Returns the ReadPreference TagSet set for the connection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getReadPrefTagSet)

## `db.getMongo().getReadPref()`

Returns the ReadPreference set for the connection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getReadPref)

## `db.getMongo().getReadConcern()`

Returns the ReadConcern set for the connection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getReadConcern)

## `db.getMongo().getWriteConcern()`

Returns the WriteConcern set for the connection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.getWriteConcern)

## `db.getMongo().setReadPref(mode, tagSets, hedgeOptions)`

Sets the ReadPreference for the connection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.setReadPref)

## `db.getMongo().setReadConcern(level)`

Sets the ReadConcern for the connection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.setReadConcern)

## `db.getMongo().setWriteConcern('majority')`

Sets the WriteConcern for the connection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.setWriteConcern)

## `const cursor = db.getMongo().watch(pipeline, options)`

Opens a change stream cursor on the connection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Mongo.watch)
