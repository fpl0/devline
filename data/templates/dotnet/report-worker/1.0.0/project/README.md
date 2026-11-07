# __PROJECT_NAME__

A .NET hosted worker scans a directory of text files and writes a JSON report
with file and byte counts. The worker publishes the report atomically and
logs structured counts on each run.

Run `./test-offline.sh` to build and execute one fixture job. Run
`dotnet run --project ReportWorker.csproj` to scan `inputs` on a timer and
write `reports/latest.json`. Set `Reports__RunOnce=true` for one scan. Edit
the `Reports` section in `appsettings.json` or use `Reports__...` environment
variables to change paths or interval.

Requires the .NET 10 SDK with the ASP.NET Core shared framework. The
package source list is empty because hosting, options, configuration,
structured logging, and JSON come from the installed runtime.
