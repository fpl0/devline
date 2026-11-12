# Order Ledger

A small inventory and order service built with FastAPI, Pydantic, and SQLite. Add products, reserve stock when an order is placed, fulfil or cancel the order, and inspect the sales report and audit trail. Prices and totals use integer cents in EUR.

## Run and try the workflow

Run `./test-offline.sh`, then `./run.sh`. Open `http://127.0.0.1:7441/docs`; the interactive API documentation and its assets are bundled locally. In generated projects, use `./devline.sh test` and `./devline.sh run`.

```sh
curl -s http://127.0.0.1:7441/products -H 'Content-Type: application/json' \
  -d '{"sku":"BOOK","name":"Field Notes","price_cents":1299,"stock":5}'
curl -s http://127.0.0.1:7441/orders -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: first-order' \
  -d '{"customer":"Alex","lines":[{"sku":"BOOK","quantity":2}]}'
```

Copy the returned order `id`. `GET /orders/ID` shows the price snapshot and history. `PATCH /orders/ID` with `{"status":"fulfilled"}` completes it; `{"status":"cancelled"}` releases its reserved stock. A closed order cannot change to another terminal state. Repeating the same transition is safe.

- `POST /products/SKU/stock` with `{"quantity":10}` replenishes a product.
- `GET /orders?status=placed&limit=50&offset=0` lists orders.
- `GET /reports/sales` shows totals by status and products with fewer than five units.
- Reuse an `Idempotency-Key` only for an identical request. A changed request returns 409.

The database is `data/ledger.sqlite3`; set `LEDGER_DATABASE` to choose another path. Restarting preserves products, orders, and history. The service binds to loopback and is a local single-workspace application without user accounts.

## Design

`app.py` owns HTTP validation and response semantics. `ledger.py` owns domain operations and database transactions. SQLite's write transaction reserves all order lines together; an invalid line or insufficient stock rolls the whole order back. The database stores item names and prices at purchase time. An idempotency key and request fingerprint make retried submissions safe.

`smoke.py` tests the complete order lifecycle, restart persistence, cancellation replay, failed-order rollback, concurrent attempts to oversell, validation, and offline documentation. `test-offline.sh` installs only the pinned local wheels.

To extend the app, add tax and shipping rules to the domain layer and test their rounding boundaries before exposing them through the API. Add a schema migration when changing stored fields. The current schema is deliberately a compact local order model; it does not implement payment processing.
