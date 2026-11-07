# Query cursor methods

Collection Cursor

- `addOption` — Adds OP_QUERY wire protocol flags, such as the tailable flag, to change the behavior of queries. Accepts: DBQuery.Option fields tailable, slaveOk, noTimeout, awaitData, exhaust, partial.

- `allowDiskUse` — Sets the 'allowDiskUse' option. If no argument is passed, the default is true.

- `allowPartialResults` — Sets the 'partial' option to true.

- `collation` — Specifies the collation for the cursor returned by the db.collection.find(). To use, append to the db.collection.find().

- `comment` — Adds a comment field to the query.

- `count` — Counts the number of documents referenced by a cursor.

- `hasNext` — cursor.hasNext() returns true if the cursor returned by the db.collection.find() query can iterate further to return more documents. NOTE: if the cursor is tailable with awaitData then hasNext will block until a document is returned. To check if a document is in the cursor's batch without waiting, use tryNext instead

- `hint` — Call this method on a query to override MongoDB’s default index selection and query optimization process. Use db.collection.getIndexes() to return the list of current indexes on a collection.

- `limit` — Use the limit() method on a cursor to specify the maximum number of documents the cursor will return.

- `max` — Specifies the exclusive upper bound for a specific index in order to constrain the results of find(). max() provides a way to specify an upper bound on compound key indexes.

- `maxAwaitTimeMS` — Set a maxAwaitTimeMS on a tailing cursor query to allow to customize the timeout value for the option awaitData (Only supported on MongoDB 3.2 or higher, ignored otherwise)

- `min` — Specifies the inclusive lower bound for a specific index in order to constrain the results of find(). min() provides a way to specify lower bounds on compound key indexes.

- `next` — The next document in the cursor returned by the db.collection.find() method. NOTE: if the cursor is tailable with awaitData then hasNext will block until a document is returned. To check if a document is in the cursor's batch without waiting, use tryNext instead

- `noCursorTimeout` — Instructs the server to avoid closing a cursor automatically after a period of inactivity.

- `oplogReplay` — Sets oplogReplay cursor flag to true.

- `readPref` — Append readPref() to a cursor to control how the client routes the query to members of the replica set.

- `returnKey` — Modifies the cursor to return index keys rather than the documents.

- `size` — A count of the number of documents that match the db.collection.find() query after applying any cursor.skip() and cursor.limit() methods.

- `tailable` — Marks the cursor as tailable.

- `maxScan` — deprecated, non-functional

- `showRecordId` — Modifies the output of a query by adding a field $recordId to matching documents. $recordId is the internal key which uniquely identifies a document in a collection.

- `readConcern` — Specify a read concern for the db.collection.find() method.

## `db.collection.find(query, projection, options).addOption(flag)`

Adds OP_QUERY wire protocol flags, such as the tailable flag, to change the behavior of queries. Accepts: DBQuery.Option fields tailable, slaveOk, noTimeout, awaitData, exhaust, partial.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.addOption)

## `db.collection.find(query, projection, options).sort(sort).allowDiskUse(false)`

Sets the 'allowDiskUse' option. If no argument is passed, the default is true.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.allowDiskUse)

## `db.collection.find(query, projection, options).allowPartialResults()`

Sets the 'partial' option to true.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.allowPartialResults)

## `db.collection.find(query, projection, options).collation(collationDocument)`

Specifies the collation for the cursor returned by the db.collection.find(). To use, append to the db.collection.find().

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.collation)

## `db.collection.find(query, projection, options).comment('Comment')`

Adds a comment field to the query.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.comment)

## `db.collection.find(query, projection, options).count()`

Counts the number of documents referenced by a cursor.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.count)

## `db.collection.find(query, projection, options).hasNext()`

cursor.hasNext() returns true if the cursor returned by the db.collection.find() query can iterate further to return more documents. NOTE: if the cursor is tailable with awaitData then hasNext will block until a document is returned. To check if a document is in the cursor's batch without waiting, use tryNext instead

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.hasNext)

## `db.collection.find(query, projection, options).hint(index)`

Call this method on a query to override MongoDB’s default index selection and query optimization process. Use db.collection.getIndexes() to return the list of current indexes on a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.hint)

## `db.collection.find(query, projection, options).limit(2)`

Use the limit() method on a cursor to specify the maximum number of documents the cursor will return.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.limit)

## `db.collection.find(query, projection, options).max(indexBoundsDocument)`

Specifies the exclusive upper bound for a specific index in order to constrain the results of find(). max() provides a way to specify an upper bound on compound key indexes.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.max)

## `db.collection.find(query, projection, options).maxAwaitTimeMS(timeLimit)`

Set a maxAwaitTimeMS on a tailing cursor query to allow to customize the timeout value for the option awaitData (Only supported on MongoDB 3.2 or higher, ignored otherwise)

## `db.collection.find(query, projection, options).min(indexBoundsDocument)`

Specifies the inclusive lower bound for a specific index in order to constrain the results of find(). min() provides a way to specify lower bounds on compound key indexes.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.min)

## `db.collection.find(query, projection, options).next()`

The next document in the cursor returned by the db.collection.find() method. NOTE: if the cursor is tailable with awaitData then hasNext will block until a document is returned. To check if a document is in the cursor's batch without waiting, use tryNext instead

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.next)

## `db.collection.find(query, projection, options).noCursorTimeout()`

Instructs the server to avoid closing a cursor automatically after a period of inactivity.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.noCursorTimeout)

## `db.collection.find(query, projection, options).oplogReplay()`

Sets oplogReplay cursor flag to true.

## `db.collection.find(query, projection, options).readPref(mode, tagSet)`

Append readPref() to a cursor to control how the client routes the query to members of the replica set.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.readPref)

## `db.collection.find(query, projection, options).returnKey()`

Modifies the cursor to return index keys rather than the documents.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.returnKey)

## `db.collection.find(query, projection, options).size()`

A count of the number of documents that match the db.collection.find() query after applying any cursor.skip() and cursor.limit() methods.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.size)

## `db.collection.find(query, projection, options).tailable({ awaitData: true })`

Marks the cursor as tailable.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.tailable)

## `db.collection.find(query, projection, options).showRecordId`

Modifies the output of a query by adding a field $recordId to matching documents. $recordId is the internal key which uniquely identifies a document in a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.showRecordId)

## `db.collection.find(query, projection, options).readConcern(level)`

Specify a read concern for the db.collection.find() method.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/cursor.readConcern)
