# __PROJECT_NAME__

An ASP.NET Core API backed by EF Core and SQLite. Requires .NET SDK 10.0.400 or a later patch in the same feature band.

Run `./test-offline.sh` to restore from the included local NuGet feed and exercise the database. To run the API after restoring, use `dotnet run --no-restore`.

## Included local packages

The bundled NuGet feed includes EF Core and SQLite, Npgsql for PostgreSQL,
MySqlConnector, StackExchange.Redis, RabbitMQ.Client, the AWS S3 SDK, ASP.NET
Core JWT bearer authentication, and xUnit. ASP.NET Core includes HTTP clients
and structured logging in the shared framework. Exact package versions and
transitive dependencies are recorded in the two `packages.lock.json` files.

`./test-offline.sh` restores against the local feed, writes and reads an
in-memory SQLite record, and exercises client configuration, S3 request
signing, and JWT verification. Remote service integration requires service
endpoints and credentials. Configure local services in Devline when needed.
