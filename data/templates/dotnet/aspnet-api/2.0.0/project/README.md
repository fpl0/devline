# ASP.NET Core Task API

A controller-based JSON API for listing, creating, updating, and deleting tasks. EF Core stores them in `tasks.db`, so they survive a restart. The project includes its locked NuGet feed and needs no service containers.

```sh
./devline.sh test
./devline.sh run
```

In a second terminal, run:

```sh
curl -fsS http://127.0.0.1:5080/health/ready
curl -fsSi http://127.0.0.1:5080/api/tasks \
  -H 'Content-Type: application/json' \
  -d '{"title":"Read the offline docs","completed":false}'
curl -fsS http://127.0.0.1:5080/api/tasks
```

The POST returns `201 Created` and a `Location` for the new task. Send a `GET`, `PUT` with a title and completion flag, or `DELETE` to that URL. Invalid input returns a validation problem with status 400. An unknown ID returns 404. The [HTTP examples](TaskApi.http) contain the whole sequence.

The app binds to loopback. Set `ConnectionStrings__Tasks` to use a different SQLite file. The app creates the demo schema on first run. The smoke test uses a disposable database and checks persistence across an app restart.
