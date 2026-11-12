# Incident Room

A live incident-response workspace built with Phoenix Channels and OTP. Open incidents, assign an owner and severity, move through investigation and recovery, and share a timestamped response timeline across browser windows. Local state survives server restarts.

## Run

```sh
./test-offline.sh
./run.sh
```

Open `http://127.0.0.1:4000` in two browser windows. Enter a name in each, open an incident, and post updates. The second window updates immediately through a WebSocket. Identify the cause, move to monitoring, then resolve the incident. Include resolved incidents to inspect or reopen them.

Every update includes the incident revision seen by the browser. If another person has changed it, a stale submission is rejected with a message asking you to review the latest state. Invalid status transitions, blank names, and oversized messages are rejected by the server.

The journal is `data/incidents.json`; set `INCIDENT_DATABASE` to choose another path. The endpoint binds to loopback and checks WebSocket origins against its configured local port. Names identify timeline entries but are not authenticated identities; this is one trusted local workspace.

## Design

- `IncidentRoom.Room` is the GenServer that serializes changes. It validates the domain transition, writes the next journal atomically, then broadcasts the saved snapshot.
- `IncidentRoom.Channel` handles the `create` and `update` protocol and forwards PubSub notifications to connected browsers.
- `DevlinePhoenix.Endpoint` runs through Bandit and serves local assets. `GET /incidents` exposes the current snapshot; `GET /healthz` supplies readiness.
- `priv/static/app.js` uses Phoenix's official JavaScript client. It renders user content through text nodes. The client module is copied from the bundled Phoenix distribution; its upstream license remains in `vendor/phoenix/LICENSE.md`.

Tests exercise valid and rejected transitions, optimistic revisions, concurrent incident creation, restart persistence, the HTTP health endpoint, and two subscribed channels receiving the same persisted update. Hex dependencies are vendored; no npm build or external service is needed.

The journal is intentionally owned by a single running node. For a shared deployment, move storage into a transactional database and add authentication and authorization before exposing the server. Keep broadcasts after successful persistence so clients never observe an update that failed to save.
