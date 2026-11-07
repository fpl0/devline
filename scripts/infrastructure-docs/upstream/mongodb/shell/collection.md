# Collection methods

Collection Class

- `aggregate` — Calculates aggregate values for the data in a collection or a view.

- `bulkWrite` — Performs multiple write operations with controls for order of execution.

- `count` — Returns the count of documents that would match a find() query for the collection or view.

- `countDocuments` — Returns the count of documents that match the query for a collection or view.

- `deleteMany` — Removes all documents that match the filter from a collection.

- `deleteOne` — Removes a single document from a collection.

- `distinct` — Finds the distinct values for a specified field across a single collection or view and returns the results in an array.

- `estimatedDocumentCount` — Returns the count of all documents in a collection or view.

- `find` — Selects documents in a collection or view.

- `findAndModify` — Modifies and returns a single document.

- `findOne` — Selects documents in a collection or view.

- `renameCollection` — Renames a collection.

- `findOneAndDelete` — Deletes a single document based on the filter and sort criteria, returning the deleted document.

- `findOneAndReplace` — Modifies and replaces a single document based on the filter and sort criteria.

- `findOneAndUpdate` — Updates a single document based on the filter and sort criteria.

- `insert` — Inserts a document or documents into a collection.

- `insertMany` — Inserts multiple documents into a collection.

- `insertOne` — Inserts a document into a collection.

- `isCapped` — Checks if a collection is capped

- `remove` — Removes documents from a collection.

- `replaceOne` — Replaces a single document within the collection based on the filter.

- `update` — Modifies an existing document or documents in a collection.

- `updateMany` — Updates all documents that match the specified filter for a collection.

- `updateOne` — Updates a single document within the collection based on the filter.

- `compactStructuredEncryptionData` — Compacts structured encryption data

- `convertToCapped` — calls {convertToCapped:'coll', size:maxBytes}} command

- `createIndexes` — Creates one or more indexes on a collection

- `createIndex` — Creates one index on a collection

- `ensureIndex` — Creates one index on a collection

- `getIndexes` — Returns an array that holds a list of documents that identify and describe the existing indexes on the collection.

- `getIndexSpecs` — Alias for getIndexes. Returns an array that holds a list of documents that identify and describe the existing indexes on the collection.

- `getIndices` — Alias for getIndexes. Returns an array that holds a list of documents that identify and describe the existing indexes on the collection.

- `getIndexKeys` — Return an array of key patterns for indexes defined on collection

- `dropIndexes` — Drops the specified index or indexes (except the index on the _id field) from a collection.

- `dropIndex` — Drops or removes the specified index from a collection.

- `totalIndexSize` — Reports the total size used by the indexes on a collection.

- `reIndex` — Rebuilds all existing indexes on a collection.

- `getDB` — Get current database.

- `getMongo` — Returns the Mongo object.

- `dataSize` — This method provides a wrapper around the size output of the collStats (i.e. db.collection.stats()) command.

- `storageSize` — The total amount of storage allocated to this collection for document storage.

- `totalSize` — The total size in bytes of the data in the collection plus the size of every index on the collection.

- `drop` — Removes a collection or view from the database.

- `exists` — Returns collection infos if the collection exists or null otherwise.

- `getFullName` — Returns the name of the collection prefixed with the database name.

- `getName` — Returns the name of the collection.

- `runCommand` — Runs a db command with the given name where the first param is the collection name.

- `explain` — Returns information on the query plan.

- `stats` — Returns statistics about the collection.

- `latencyStats` — returns the $latencyStats aggregation for the collection. Takes an options document with an optional boolean 'histograms' field.

- `initializeOrderedBulkOp` — Initializes an ordered bulk command. Returns an instance of Bulk

- `initializeUnorderedBulkOp` — Initializes an unordered bulk command. Returns an instance of Bulk

- `getPlanCache` — Returns an interface to access the query plan cache for a collection. The interface provides methods to view and clear the query plan cache.

- `mapReduce` — Calls the mapReduce command

- `validate` — Calls the validate command. Default full value is false

- `getShardVersion` — Calls the getShardVersion command

- `getShardDistribution` — Prints the data distribution statistics for a sharded collection.

- `getShardLocation` — Returns a document containing the shards where this collection is located as well as whether the collection itself is sharded.

- `watch` — Opens a change stream cursor on the collection

- `hideIndex` — Hides an existing index from the query planner.

- `unhideIndex` — Unhides an existing index from the query planner.

- `analyzeShardKey` — Returns metrics for evaluating a shard key. That is, ‘key’ can be a candidate shard key for an unsharded or sharded collection, or the current shard key for a sharded collection.

- `configureQueryAnalyzer` — Starts or stops collecting metrics about reads and writes against an unsharded or sharded collection.

- `checkMetadataConsistency` — Returns a cursor with information about metadata inconsistencies

- `getSearchIndexes` — Returns an array that holds a list of documents that identify and describe the existing search indexes on the collection.

- `createSearchIndex` — Creates one search indexes on a collection

- `createSearchIndexes` — Creates one or more search indexes on a collection

- `dropSearchIndex` — Drops or removes the specified search index from a collection.

- `updateSearchIndex` — Updates the sepecified search index.

## `db.collection.aggregate(pipeline, options)`

Calculates aggregate values for the data in a collection or a view.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.aggregate)

## `db.collection.bulkWrite(operations, options)`

Performs multiple write operations with controls for order of execution.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.bulkWrite)

## `db.collection.count(query, options)`

Returns the count of documents that would match a find() query for the collection or view.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.count)

## `db.collection.countDocuments(query, options)`

Returns the count of documents that match the query for a collection or view.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.countDocuments)

## `db.collection.deleteMany()`

Removes all documents that match the filter from a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.deleteMany)

## `db.collection.deleteOne(filter, options)`

Removes a single document from a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.deleteOne)

## `db.collection.distinct(field, query, options)`

Finds the distinct values for a specified field across a single collection or view and returns the results in an array.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.distinct)

## `db.collection.estimatedDocumentCount(options)`

Returns the count of all documents in a collection or view.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.estimatedDocumentCount)

## `db.collection.find(query, projection, options)`

Selects documents in a collection or view.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.find)

## `db.collection.findAndModify(spec)`

Modifies and returns a single document.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.findAndModify)

## `db.collection.findOne(query, projection, options)`

Selects documents in a collection or view.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.findOne)

## `db.coll.renameCollection("newName")`

Renames a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.renameCollection/)

## `db.collection.findOneAndDelete(filter, options)`

Deletes a single document based on the filter and sort criteria, returning the deleted document.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.findOneAndDelete)

## `db.collection.findOneAndReplace(filter, replacement, options)`

Modifies and replaces a single document based on the filter and sort criteria.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.findOneAndReplace)

## `db.collection.findOneAndUpdate(filter, update, options)`

Updates a single document based on the filter and sort criteria.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.findOneAndUpdate)

## `db.collection.insert(document, options)`

Inserts a document or documents into a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.insert)

## `db.collection.insertMany(documents, options)`

Inserts multiple documents into a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.insertMany)

## `db.collection.insertOne(document, options)`

Inserts a document into a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.insertOne)

## `db.collection.isCapped()`

Checks if a collection is capped

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.isCapped)

## `db.collection.remove(query, options)`

Removes documents from a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.remove)

## `db.collection.replaceOne(filter, replacement, options)`

Replaces a single document within the collection based on the filter.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.replaceOne)

## `db.collection.update(query, update, options)`

Modifies an existing document or documents in a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.update)

## `db.collection.updateMany(filter, update, options)`

Updates all documents that match the specified filter for a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.updateMany)

## `db.collection.updateOne(filter, update, options)`

Updates a single document within the collection based on the filter.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.updateOne)

## `db.collection.compactStructuredEncryptionData()`

Compacts structured encryption data

## `db.coll.convertToCapped(10000)`

calls {convertToCapped:'coll', size:maxBytes}} command

## `db.coll.createIndexes([{ category: 1 }], { name: 'index-1' })`

Creates one or more indexes on a collection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.createIndexes)

## `db.coll.createIndex({ category: 1 }, { name: 'index-1' })`

Creates one index on a collection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.createIndex)

## `db.coll.ensureIndex({ category: 1 }, { name: 'index-1' })`

Creates one index on a collection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.ensureIndex)

## `db.collection.getIndexes()`

Returns an array that holds a list of documents that identify and describe the existing indexes on the collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getIndexes)

## `db.collection.getIndexSpecs()`

Alias for getIndexes. Returns an array that holds a list of documents that identify and describe the existing indexes on the collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getIndexes)

## `db.collection.getIndices()`

Alias for getIndexes. Returns an array that holds a list of documents that identify and describe the existing indexes on the collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getIndexes)

## `db.collection.getIndexKeys()`

Return an array of key patterns for indexes defined on collection

## `db.collection.dropIndexes()`

Drops the specified index or indexes (except the index on the _id field) from a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.dropIndexes)

## `db.collection.dropIndex()`

Drops or removes the specified index from a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.dropIndex)

## `db.collection.totalIndexSize()`

Reports the total size used by the indexes on a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.totalIndexSize)

## `db.collection.getDB()`

Get current database.

## `db.collection.getMongo()`

Returns the Mongo object.

## `db.collection.dataSize()`

This method provides a wrapper around the size output of the collStats (i.e. db.collection.stats()) command.

[MongoDB method reference](hhttps://mongodb.com/docs/manual/reference/method/db.collection.dataSize)

## `db.collection.storageSize()`

The total amount of storage allocated to this collection for document storage.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.storageSize)

## `db.collection.totalSize()`

The total size in bytes of the data in the collection plus the size of every index on the collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.totalSize)

## `db.students.drop()`

Removes a collection or view from the database.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.drop)

## `db.coll.exists()`

Returns collection infos if the collection exists or null otherwise.

## `db.coll.getFullName()`

Returns the name of the collection prefixed with the database name.

## `db.coll.getName()`

Returns the name of the collection.

## `db.collection.runCommand("text", { search: "searchKeywords" })`

Runs a db command with the given name where the first param is the collection name.

## `db.collection.explain().<method(...)>`

Returns information on the query plan.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.explain)

## `db.collection.stats(options)`

Returns statistics about the collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.stats)

## `db.latencyStats({ histograms: true })`

returns the $latencyStats aggregation for the collection. Takes an options document with an optional boolean 'histograms' field.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.latencyStats)

## `db.coll.initializeOrderedBulkOp()`

Initializes an ordered bulk command. Returns an instance of Bulk

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.initializeOrderedBulkOp)

## `db.coll.initializeUnorderedBulkOp()`

Initializes an unordered bulk command. Returns an instance of Bulk

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.initializeUnorderedBulkOp)

## `db.coll.getPlanCache()`

Returns an interface to access the query plan cache for a collection. The interface provides methods to view and clear the query plan cache.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getPlanCache/)

## `db.coll.mapReduce(mapFn, reduceFn, options)`

Calls the mapReduce command

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.mapReduce)

## `db.coll.validate(<full>)`

Calls the validate command. Default full value is false

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.validate)

## `db.coll.getShardVersion()`

Calls the getShardVersion command

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getShardVersion)

## `db.coll.getShardDistribution()`

Prints the data distribution statistics for a sharded collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getShardDistribution)

## `db.coll.getShardLocation()`

Returns a document containing the shards where this collection is located as well as whether the collection itself is sharded.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getShardLocation)

## `const cursor = db.collection.watch(pipeline, options)`

Opens a change stream cursor on the collection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.watch)

## `db.collection.hideIndex("index_1")`

Hides an existing index from the query planner.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.hideIndex)

## `db.collection.unhideIndex("index_1")`

Unhides an existing index from the query planner.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.unhideIndex)

## `db.coll.analyzeShardKey(key)`

Returns metrics for evaluating a shard key. That is, ‘key’ can be a candidate shard key for an unsharded or sharded collection, or the current shard key for a sharded collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.analyzeShardKey)

## `db.coll.configureQueryAnalyzer(options)`

Starts or stops collecting metrics about reads and writes against an unsharded or sharded collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.configureQueryAnalyzer)

## `db.coll.checkMetadataConsistency(<options>)`

Returns a cursor with information about metadata inconsistencies

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.checkMetadataConsistency)

## `db.coll.getSearchIndexes(<name>)`

Returns an array that holds a list of documents that identify and describe the existing search indexes on the collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.getSearchIndexes)

## `db.coll.createSearchIndex(<name>, definition)`

Creates one search indexes on a collection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.createSearchIndex)

## `db.coll.createSearchIndexes(specs)`

Creates one or more search indexes on a collection

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.createSearchIndexes)

## `db.coll.dropSearchIndex(name)`

Drops or removes the specified search index from a collection.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.dropSearchIndex)

## `db.coll.updateSearchIndex(name, definition)`

Updates the sepecified search index.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/db.collection.updateSearchIndex)
