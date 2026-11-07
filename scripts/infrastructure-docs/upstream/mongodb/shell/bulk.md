# Bulk write methods

Bulk operations builder used to construct a list of write operations to perform in bulk for a single collection. To instantiate the builder, use either the db.collection.initializeOrderedBulkOp() or the db.collection.initializeUnorderedBulkOp() method.

- `execute` — Executes the bulk operation.

- `find` — Adds a find to the bulk operation.

- `insert` — Adds an insert to the bulk operation.

- `toString` — Returns as a string a JSON document that contains the number of operations and batches in the Bulk() object.

- `getOperations` — Returns the batches executed by the bulk write.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Bulk)

## `bulkOp.execute()`

Executes the bulk operation.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Bulk.execute/)

## `bulkOp.find(<filter>)`

Adds a find to the bulk operation.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Bulk.find/)

## `db.insert(<document>)`

Adds an insert to the bulk operation.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Bulk.insert/)

## `bulkOp.toString()`

Returns as a string a JSON document that contains the number of operations and batches in the Bulk() object.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Bulk.toString/)

## `bulkOp.getOperations()`

Returns the batches executed by the bulk write.

[MongoDB method reference](https://mongodb.com/docs/manual/reference/method/Bulk.getOperations/)
