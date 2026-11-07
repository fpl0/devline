# Use MongoDB in Devline

**Image** `docker.io/library/mongo:8.0.34-noble` for Linux arm64. **Default host ports** `27017`.

## Connect and work with a document

The prepared instance is standalone. Use the connection URI from Devline, which authenticates the initial admin user against `admin` and selects the application database `devline`.

```sh
docker exec "$CONTAINER" sh -c 'exec mongosh --quiet "mongodb://devline:$MONGO_INITDB_ROOT_PASSWORD@127.0.0.1:27017/devline?authSource=admin" --eval "db.probe.updateOne({id:1},{\$set:{value:\"offline\"}},{upsert:true}); printjson(db.probe.findOne({id:1}))"'
```

When testing application privileges, create a separate database-scoped user. The bootstrap credentials are for local administration. Test replica set transactions separately because a standalone instance does not exercise them.

## Back up and restore

Use the MongoDB Database Tools inside the prepared container. The dump streams to a file on your Mac.

```sh
docker exec "$CONTAINER" mongodump --version
docker exec "$CONTAINER" sh -c 'exec mongodump --username=devline --password="$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase=admin --db=devline --archive --gzip' > devline.mongo.archive
docker exec -i "$CONTAINER" sh -c 'exec mongorestore --username=devline --password="$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase=admin --archive --gzip --nsFrom="devline.*" --nsTo="devline_restore.*"' < devline.mongo.archive
```

Pause writers for a consistent logical copy of this standalone database. For a local clone, stop the server and use the [cold volume backup](portable-toolbox.md) on its `/data/db` volume and any configuration volume you use. Reopen with the same server version and verify a document read.

[MongoDB 8.0 manual](https://www.mongodb.com/docs/v8.0/) · [MongoDB Database Tools](https://www.mongodb.com/docs/database-tools/)
