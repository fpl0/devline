"""A small analytical warehouse with explicit validation and repeatable ingestion."""

import csv
from datetime import date
import hashlib
import html
import io
import json
from pathlib import Path
from string import Template

import duckdb

FIELDS = {"order_id", "date", "category", "quantity", "unit_price_cents"}


def connect(path):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    database = duckdb.connect(
        str(path),
        config={
            "threads": "2",
            "autoinstall_known_extensions": "false",
            "autoload_known_extensions": "false",
        },
    )
    database.execute("""CREATE TABLE IF NOT EXISTS sales (
        order_id VARCHAR PRIMARY KEY, sold_on DATE NOT NULL, category VARCHAR NOT NULL,
        quantity BIGINT NOT NULL CHECK(quantity>0), unit_price_cents BIGINT NOT NULL CHECK(unit_price_cents>=0));
        CREATE TABLE IF NOT EXISTS imports (
        digest VARCHAR PRIMARY KEY, source VARCHAR NOT NULL, accepted INTEGER NOT NULL,
        rejected INTEGER NOT NULL, duplicates INTEGER NOT NULL, imported_at TIMESTAMP DEFAULT current_timestamp);
    """)
    return database


def parse_row(row):
    if set(row) != FIELDS or None in row.values():
        raise ValueError("Expected exactly: " + ", ".join(sorted(FIELDS)))
    identity, category = row["order_id"].strip(), row["category"].strip()
    if not identity or len(identity) > 80 or not category or len(category) > 80:
        raise ValueError("order_id and category must contain 1–80 characters")
    sold_on = date.fromisoformat(row["date"])
    quantity, price = int(row["quantity"]), int(row["unit_price_cents"])
    if not 1 <= quantity <= 1_000_000 or not 0 <= price <= 100_000_000:
        raise ValueError("quantity must be 1–1000000 and unit_price_cents 0–100000000")
    return identity, sold_on, category, quantity, price


def ingest(database, source):
    source = Path(source)
    raw = source.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    previous = database.execute(
        "SELECT accepted,rejected,duplicates FROM imports WHERE digest=?", [digest]
    ).fetchone()
    if previous:
        return {
            "accepted": 0,
            "rejected": 0,
            "duplicates": 0,
            "already_imported": True,
            "errors": [],
        }
    # Decode and inspect all CSV rows before starting the transaction: malformed files never partly import.
    text = raw.decode("utf-8-sig")
    reader = csv.DictReader(io.StringIO(text), strict=True)
    if (
        reader.fieldnames is None
        or len(reader.fieldnames) != len(FIELDS)
        or set(reader.fieldnames) != FIELDS
    ):
        raise ValueError(
            "CSV headers must contain each required column exactly once: "
            + ", ".join(sorted(FIELDS))
        )
    rows = list(reader)
    if not rows:
        raise ValueError("Input needs a header and at least one data row")
    accepted = duplicates = 0
    errors = []
    database.execute("BEGIN TRANSACTION")
    try:
        for line, row in enumerate(rows, 2):
            try:
                values = parse_row(row)
                existing = database.execute(
                    "SELECT * FROM sales WHERE order_id=?", [values[0]]
                ).fetchone()
                if existing:
                    if tuple(existing) != values:
                        raise ValueError(
                            "order_id already exists with different values"
                        )
                    duplicates += 1
                    continue
                database.execute("INSERT INTO sales VALUES (?,?,?,?,?)", values)
                accepted += 1
            except (ValueError, TypeError) as error:
                errors.append({"line": line, "reason": str(error), "row": row})
        database.execute(
            "INSERT INTO imports(digest,source,accepted,rejected,duplicates) VALUES (?,?,?,?,?)",
            [digest, source.name, accepted, len(errors), duplicates],
        )
        database.execute("COMMIT")
    except Exception:
        database.execute("ROLLBACK")
        raise
    return {
        "accepted": accepted,
        "rejected": len(errors),
        "duplicates": duplicates,
        "already_imported": False,
        "errors": errors,
    }


def report(database, since=None, until=None):
    start = date.fromisoformat(since) if since else date.min
    end = date.fromisoformat(until) if until else date.max
    if start > end:
        raise ValueError("--since must be on or before --until")
    rows = database.execute(
        """SELECT category,count(*) AS orders,sum(quantity) AS units,
        sum(quantity*unit_price_cents)::BIGINT AS revenue_cents
        FROM sales WHERE sold_on BETWEEN ? AND ? GROUP BY category ORDER BY revenue_cents DESC,category""",
        [start, end],
    ).fetchall()
    monthly = database.execute(
        """SELECT strftime(sold_on,'%Y-%m') AS month,
        sum(quantity*unit_price_cents)::BIGINT AS revenue_cents FROM sales
        WHERE sold_on BETWEEN ? AND ? GROUP BY month ORDER BY month""",
        [start, end],
    ).fetchall()
    return {
        "currency": "EUR",
        "since": since,
        "until": until,
        "categories": [
            dict(zip(["category", "orders", "units", "revenue_cents"], row))
            for row in rows
        ],
        "months": [dict(zip(["month", "revenue_cents"], row)) for row in monthly],
        "total_cents": sum(row[3] for row in rows),
        "imports": database.execute("SELECT count(*) FROM imports").fetchone()[0],
    }


def render_html(data):
    def euros(cents):
        return f"€{cents // 100:,}.{cents % 100:02d}"

    rows = "".join(
        f"<tr><th scope='row'>{html.escape(row['category'])}</th><td>{row['orders']}</td>"
        f"<td>{row['units']}</td><td>{euros(row['revenue_cents'])}</td></tr>"
        for row in data["categories"]
    )
    months = "".join(
        f"<li>{row['month']}: {euros(row['revenue_cents'])}</li>"
        for row in data["months"]
    )
    template = Template(
        (Path(__file__).parent / "templates/report.html").read_text(encoding="utf-8")
    )
    return template.substitute(
        since=html.escape(data["since"] or "All dates"),
        until=html.escape(data["until"] or "latest"),
        total=euros(data["total_cents"]),
        rows=rows,
        months=months,
        imports=data["imports"],
    )
