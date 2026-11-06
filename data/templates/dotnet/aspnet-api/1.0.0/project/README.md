# __PROJECT_NAME__

An ASP.NET Core API backed by EF Core and SQLite. Requires .NET SDK 10.0.400 or a later patch in the same feature band.

Run `./test-offline.sh` to restore from the included local NuGet feed and exercise the database. To run the API after restoring, use `dotnet run --no-restore`.
