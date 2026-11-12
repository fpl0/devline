# Purchase Desk

A complete local purchase approval workflow: submit a request with an exact EUR amount and business reason, review the queue, approve or reject with a note, and inspect the permanent history. Requests and decisions survive app restarts.

```sh
./run.sh
./build-app.sh
./test-offline.sh
```

Open the address printed by the server (Java: http://127.0.0.1:8081; .NET: http://127.0.0.1:5081). Sign in as **alice** with password **devline-local**. Submit “Workshop materials”, EUR 240.50, with a business reason. Sign out and sign in as **bob**, using the same password. Open the request and approve it with a note. Filter the queue to Approved and inspect the history.

Alice is a requester; Bob is an approver and can also submit requests. Nobody may decide their own request. Decisions are final, submitted amounts cannot be edited, and stale forms return a conflict instead of overwriting a decision. Both the state change and audit event are committed in one transaction. Form submissions use CSRF protection; user text is HTML-escaped.

These are fixed local demonstration identities, not a user-management system. The app binds to loopback and requires no external services. Replace the demo identity provider before exposing it to a network. Data is stored in `data/`; stop the app before backing it up. EUR values are stored as integer cents, with limits of 0.01–1,000,000.00.

The test launches the real server against a temporary database, submits rendered forms, verifies permissions and self-approval rules, checks invalid money and CSRF, confirms that decisions cannot change, and restarts the process to verify persistence. Java additionally tests the transactional domain service directly. Dependencies are bundled for offline builds.
