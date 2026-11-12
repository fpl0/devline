# Workshop Desk

A Rails application for a small repair or service workshop. Receive a job, prepare an estimate, record approval, start work, mark it ready, and record collection. Each step creates an audit entry; jobs and history survive restarts in SQLite.

## Run

```sh
./test-offline.sh
./run.sh
```

Open `http://127.0.0.1:3000`. The launcher installs bundled gems if necessary and applies database migrations. Generated projects use `./devline.sh test` and `./devline.sh run`.

1. Receive a job with its title, customer, and intake notes.
2. Open the job and **Quote** an estimate in cents (`4500` means €45.00).
3. **Approve**, **Start**, **Finish**, then **Collect** the job, adding a note at each handoff.
4. Filter the board by status or search by title/customer. Open a closed job to inspect its full history.

To add the optional example jobs, run `./devline.sh bundle exec ruby bin/rails db:seed` in a generated project. The seeds are repeatable and do not reset existing work.

A job can be cancelled before work begins. A received job cannot jump directly to collection. Two tabs editing the same job use a version check: a stale form returns a clear conflict instead of overwriting the newer update.

## Design

- `Job` owns validation and the allowed transition map. Its transaction changes status and appends history together.
- `JobEvent` records each handoff. It is presented as a timeline on the detail page.
- `JobsController` handles intake, filtered queries, and transition responses.
- ERB templates use Rails escaping, labelled forms, CSRF tokens, and responsive local CSS. No frontend build server is required.
- `db/migrate` is the schema source. Local development data lives in `storage/development.sqlite3`.

The tests cover the entire workflow, invalid transitions, invalid estimates, transactional history, stale browser forms, search, validation feedback, and HTML escaping. Tests migrate a fresh temporary database and do not change the development database.

This is a local workshop workspace without user accounts. To add multiple staff accounts, attach the actor to each `JobEvent` and authorize transitions in the controller while preserving the model's invariants. The existing customer field intentionally remains a name rather than a complete CRM record.
