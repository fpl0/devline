"""CLI for the Sales Observatory warehouse."""

import argparse
import csv
import json
from pathlib import Path
import sys

import duckdb
from observatory import connect, ingest, render_html, report


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Import validated sales and publish reproducible reports"
    )
    parser.add_argument("--database", type=Path, default=Path("data/sales.duckdb"))
    commands = parser.add_subparsers(dest="command", required=True)
    load = commands.add_parser(
        "ingest", help="Import CSV; invalid rows are reported, valid rows are committed"
    )
    load.add_argument("source", type=Path)
    load.add_argument("--rejects", type=Path)
    summary = commands.add_parser(
        "report", help="Query the warehouse and emit JSON or a standalone HTML report"
    )
    summary.add_argument("--since")
    summary.add_argument("--until")
    summary.add_argument("--html", type=Path)
    args = parser.parse_args(argv)
    try:
        with connect(args.database) as database:
            if args.command == "ingest":
                result = ingest(database, args.source)
                if args.rejects:
                    args.rejects.parent.mkdir(parents=True, exist_ok=True)
                    args.rejects.write_text(
                        json.dumps(result["errors"], indent=2) + "\n"
                    )
            else:
                result = report(database, args.since, args.until)
                if args.html:
                    args.html.parent.mkdir(parents=True, exist_ok=True)
                    args.html.write_text(render_html(result), encoding="utf-8")
            print(json.dumps(result, indent=2))
            return 2 if result.get("rejected") else 0
    except (OSError, ValueError, csv.Error, duckdb.Error) as error:
        print(f"sales: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
