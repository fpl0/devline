"""Domain operations each own a database transaction; money is integer cents."""

from contextlib import contextmanager
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sqlite3
from uuid import uuid4


class LedgerError(Exception):
    def __init__(self, code, message, status=409):
        super().__init__(message)
        self.code, self.status = code, status


class Ledger:
    def __init__(self, path):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with self.connection() as db:
            db.executescript("""
                PRAGMA journal_mode=WAL;
                CREATE TABLE IF NOT EXISTS products (
                    sku TEXT PRIMARY KEY, name TEXT NOT NULL, price_cents INTEGER NOT NULL CHECK(price_cents>0),
                    stock INTEGER NOT NULL CHECK(stock>=0));
                CREATE TABLE IF NOT EXISTS orders (
                    id TEXT PRIMARY KEY, customer TEXT NOT NULL, status TEXT NOT NULL,
                    total_cents INTEGER NOT NULL, created_at TEXT NOT NULL,
                    request_key TEXT UNIQUE NOT NULL, request_hash TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS lines (
                    order_id TEXT REFERENCES orders(id), sku TEXT REFERENCES products(sku),
                    name TEXT NOT NULL, price_cents INTEGER NOT NULL, quantity INTEGER NOT NULL,
                    PRIMARY KEY(order_id,sku));
                CREATE TABLE IF NOT EXISTS events (
                    id INTEGER PRIMARY KEY, order_id TEXT REFERENCES orders(id), status TEXT NOT NULL, at TEXT NOT NULL);
            """)

    @contextmanager
    def connection(self):
        db = sqlite3.connect(self.path, timeout=10)
        db.row_factory = sqlite3.Row
        db.execute("PRAGMA foreign_keys=ON")
        try:
            with db:
                yield db
        finally:
            db.close()

    def products(self):
        with self.connection() as db:
            return [
                dict(row) for row in db.execute("SELECT * FROM products ORDER BY sku")
            ]

    def add_product(self, data):
        try:
            with self.connection() as db:
                db.execute(
                    "INSERT INTO products VALUES (:sku,:name,:price_cents,:stock)", data
                )
        except sqlite3.IntegrityError as error:
            raise LedgerError(
                "duplicate_sku", "A product already uses this SKU."
            ) from error
        return data

    def restock(self, sku, quantity):
        with self.connection() as db:
            result = db.execute(
                "UPDATE products SET stock=stock+? WHERE sku=? RETURNING *",
                (quantity, sku),
            ).fetchone()
            if result is None:
                raise LedgerError("not_found", "Product not found.", 404)
            return dict(result)

    @staticmethod
    def _order(db, order_id):
        row = db.execute(
            "SELECT id,customer,status,total_cents,created_at FROM orders WHERE id=?",
            (order_id,),
        ).fetchone()
        if row is None:
            raise LedgerError("not_found", "Order not found.", 404)
        return dict(row) | {
            "lines": [
                dict(r)
                for r in db.execute(
                    "SELECT sku,name,price_cents,quantity FROM lines WHERE order_id=? ORDER BY sku",
                    (order_id,),
                )
            ],
            "history": [
                dict(r)
                for r in db.execute(
                    "SELECT status,at FROM events WHERE order_id=? ORDER BY id",
                    (order_id,),
                )
            ],
        }

    def order(self, order_id):
        with self.connection() as db:
            return self._order(db, order_id)

    def orders(self, status=None, limit=50, offset=0):
        with self.connection() as db:
            rows = db.execute(
                "SELECT id FROM orders WHERE (? IS NULL OR status=?) ORDER BY created_at DESC,id LIMIT ? OFFSET ?",
                (status, status, limit, offset),
            )
            return [self._order(db, row["id"]) for row in rows.fetchall()]

    def place(self, data, key):
        fingerprint = hashlib.sha256(
            json.dumps(data, sort_keys=True).encode()
        ).hexdigest()
        with self.connection() as db:
            db.execute("BEGIN IMMEDIATE")
            prior = db.execute(
                "SELECT id,request_hash FROM orders WHERE request_key=?", (key,)
            ).fetchone()
            if prior:
                if prior["request_hash"] != fingerprint:
                    raise LedgerError(
                        "idempotency_conflict",
                        "This key belongs to a different order request.",
                    )
                return self._order(db, prior["id"]), False
            skus = [line["sku"] for line in data["lines"]]
            if len(skus) != len(set(skus)):
                raise LedgerError("duplicate_line", "Use one line per SKU.", 422)
            items, total = [], 0
            for line in data["lines"]:
                product = db.execute(
                    "SELECT * FROM products WHERE sku=?", (line["sku"],)
                ).fetchone()
                if product is None:
                    raise LedgerError(
                        "unknown_sku", f"Unknown product: {line['sku']}", 422
                    )
                if product["stock"] < line["quantity"]:
                    raise LedgerError(
                        "insufficient_stock", f"Insufficient stock for {line['sku']}."
                    )
                items.append((product, line["quantity"]))
                total += product["price_cents"] * line["quantity"]
            identity, now = str(uuid4()), datetime.now(timezone.utc).isoformat()
            db.execute(
                "INSERT INTO orders VALUES (?,?,?,?,?,?,?)",
                (identity, data["customer"], "placed", total, now, key, fingerprint),
            )
            for product, quantity in items:
                db.execute(
                    "INSERT INTO lines VALUES (?,?,?,?,?)",
                    (
                        identity,
                        product["sku"],
                        product["name"],
                        product["price_cents"],
                        quantity,
                    ),
                )
                db.execute(
                    "UPDATE products SET stock=stock-? WHERE sku=?",
                    (quantity, product["sku"]),
                )
            db.execute(
                "INSERT INTO events(order_id,status,at) VALUES (?,?,?)",
                (identity, "placed", now),
            )
            return self._order(db, identity), True

    def transition(self, order_id, status):
        with self.connection() as db:
            db.execute("BEGIN IMMEDIATE")
            order = self._order(db, order_id)
            if order["status"] == status:
                return order
            if order["status"] != "placed" or status not in {"fulfilled", "cancelled"}:
                raise LedgerError(
                    "invalid_transition",
                    "Only placed orders can be fulfilled or cancelled.",
                )
            if status == "cancelled":
                for line in order["lines"]:
                    db.execute(
                        "UPDATE products SET stock=stock+? WHERE sku=?",
                        (line["quantity"], line["sku"]),
                    )
            db.execute("UPDATE orders SET status=? WHERE id=?", (status, order_id))
            db.execute(
                "INSERT INTO events(order_id,status,at) VALUES (?,?,?)",
                (order_id, status, datetime.now(timezone.utc).isoformat()),
            )
            return self._order(db, order_id)

    def report(self):
        with self.connection() as db:
            return {
                "currency": "EUR",
                "by_status": [
                    dict(r)
                    for r in db.execute(
                        "SELECT status,count(*) AS orders,coalesce(sum(total_cents),0) AS total_cents FROM orders GROUP BY status ORDER BY status"
                    )
                ],
                "low_stock": [
                    dict(r)
                    for r in db.execute(
                        "SELECT sku,name,stock FROM products WHERE stock<5 ORDER BY stock,sku"
                    )
                ],
            }
