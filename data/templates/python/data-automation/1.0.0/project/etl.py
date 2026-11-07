import argparse
import json
from pathlib import Path
from tempfile import TemporaryDirectory

import duckdb
import httpx


def summarize(source: Path) -> list[dict]:
    source = source.resolve(strict=True)
    readers = {
        ".csv": "read_csv(?, header = true, columns = {'category': 'VARCHAR', 'amount_cents': 'BIGINT'})",
        ".json": "read_json(?, columns = {'category': 'VARCHAR', 'amount_cents': 'BIGINT'})",
    }
    try:
        reader = readers[source.suffix.lower()]
    except KeyError:
        raise ValueError("Input must be a CSV or JSON file") from None
    with duckdb.connect(config={"autoinstall_known_extensions": "false", "autoload_known_extensions": "false"}) as database:
        rows = database.execute(
            f"""SELECT category, count(*) AS rows, sum(amount_cents)::BIGINT AS total_cents
                FROM {reader} GROUP BY category ORDER BY category""",
            [str(source)],
        ).fetchall()
    return [{"category": category, "rows": count, "total_cents": total} for category, count, total in rows]


def download_json(client: httpx.Client, url: str, destination: Path):
    response = client.get(url)
    response.raise_for_status()
    destination.write_text(json.dumps(response.json()), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description="Summarize integer cents by category")
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--input", type=Path, help="Local CSV or JSON path")
    source.add_argument("--url", help="Optional HTTP JSON source")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.input:
        summary = summarize(args.input)
    else:
        with TemporaryDirectory() as scratch, httpx.Client(timeout=10) as client:
            path = Path(scratch) / "source.json"
            download_json(client, args.url, path)
            summary = summarize(path)
    result = json.dumps(summary, indent=2) + "\n"
    if args.output:
        args.output.write_text(result, encoding="utf-8")
    else:
        print(result, end="")


if __name__ == "__main__":
    main()
