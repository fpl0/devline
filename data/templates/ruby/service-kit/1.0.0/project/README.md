# Dispatch Inbox

A Sinatra application for receiving payment webhooks and producing local packing slips. It demonstrates a complete integration boundary: signature verification, replay protection, durable idempotency, asynchronous processing, retry backoff, failure inspection, and manual retry.

## Run and send an event

```sh
./test-offline.sh
./run.sh
```

Open `http://127.0.0.1:7461`. In another terminal, from a generated project:

```sh
./devline.sh exec ruby send-event.rb
./devline.sh exec ruby send-event.rb examples/order-paid.json
```

The sender signs the exact bytes of the example event. The first delivery is accepted with HTTP 202; sending it again returns the existing event with HTTP 200. Reusing its `id` for different bytes returns 409.

Choose **Process due events** in the dashboard. The resulting JSON packing slip is saved in `data/packing-slips/evt-order-1042.json`. For a dedicated worker, run `./devline.sh exec ruby worker.rb --watch`. Without `--watch`, the worker handles one due event and exits. It is safe to run the dashboard and worker together: SQLite claims and generation-specific leases coordinate attempts.

## Webhook contract

`POST /webhooks/orders` requires `Content-Type: application/json`, `X-Event-Timestamp` (Unix seconds), and `X-Event-Signature` (lowercase HMAC-SHA256 hex over `timestamp.body`). Timestamps must be within five minutes. Bodies are limited to 64 KiB.

```json
{"id":"evt-order-1042","type":"order.paid","data":{"order_id":"ORDER-1042","customer":"Alex","total_cents":4890}}
```

The local signing key is created with private file permissions at `data/signing-key`. Set `DISPATCH_SECRET` to a shared secret of at least 32 bytes when using another sender. `DISPATCH_DATA` selects a different data directory; `DISPATCH_URL` changes the sender's destination. Never put the signing key in version control.

Failed delivery uses exponential backoff and stops after three attempts. The dashboard shows the error and offers **Retry event** once the underlying problem is fixed. Interrupted workers can be reclaimed after a 60-second lease. A stale worker cannot overwrite the state of a newer claim. Delivery is at least once; deterministic packing-slip filenames make this particular side effect idempotent.

## Design and tests

`lib/inbox.rb` owns Sequel transactions and processing. `lib/signature.rb` owns signing. `app.rb` is the HTTP boundary, and `public/index.html` is a small local dashboard that renders text safely. `send-event.rb` is an example integration client; `worker.rb` is a separate consumer process.

Tests cover valid signatures, expired and forged signatures, identical and conflicting duplicates, validation, retry backoff, exhausted/manual retries, restart persistence, and late completion of an expired lease. The app binds to loopback and does not implement an internet-facing administration/authentication layer. Use `./bundle-offline.sh exec ruby send-event.rb` when working directly in the source bundle.
