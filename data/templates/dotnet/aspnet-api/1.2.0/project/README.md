# Offline notes

A small ASP.NET Core app you can change and run without fetching packages. Save a note in your browser, then watch it move through RabbitMQ into the processed state. PostgreSQL stores the note before it enters the queue, so a stopped broker does not lose your work.

In Devline, add **PostgreSQL** and **RabbitMQ** to this project, then choose **Run Offline**. Devline starts the prepared local services and supplies their connection settings. Open the local URL shown in the app (normally <http://127.0.0.1:5080>).

You can also run `./run.sh` directly. With no connection settings, it uses a local SQLite file and shows “saved locally.” To connect services outside Devline, set the standard ASP.NET Core environment variables `ConnectionStrings__Postgres` (an Npgsql connection string) and `ConnectionStrings__RabbitMQ` (an AMQP URI) before launching. `ASPNETCORE_URLS` changes the listening address; `RabbitMQ__Queue` changes the durable queue name. Keep credentials out of source control.

## Change the app

- `Program.cs` defines the JSON API and live `/health/ready` endpoint.
- `Notes.cs` defines persistence and input validation. Notes awaiting publication are also the durable outbox.
- `NoteDelivery.cs` publishes with broker confirmations, consumes with manual acknowledgments, and retries outages. Repeated deliveries are harmless.
- `wwwroot/index.html` is the browser interface. It has no CDN or frontend package requirements.

This is a local development example. Schema creation uses EF Core `EnsureCreated`; add migrations when you begin evolving a lasting application database. The demo has no authentication and should listen on loopback.

## Test offline

Run `./test-offline.sh` for the unit tests. `NuGet.Config` clears remote sources and uses the bundled `nuget-packages` feed; exact app and test dependencies are locked. A local .NET 10 SDK is required. Devline detects the SDK you installed and does not install or manage it.

With the app, PostgreSQL, and RabbitMQ running, use `python3 scripts/test-services.py`. It checks the live services, creates a note through the API, and waits for confirmed publication and actual consumer processing. The command it prints can verify the same note after restarting the app and services. If RabbitMQ is stopped, `/health/ready` returns 503 and newly saved notes remain visibly pending until it returns.

The feed also includes MySQL, Redis, S3, JWT, and xUnit libraries for further experiments. These clients are available locally; the sample workflow uses PostgreSQL and RabbitMQ.
