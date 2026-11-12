import argparse
import json
from pathlib import Path
import sqlite3
import sys
from .index import Index


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Field Notes: search your Markdown and text files locally"
    )
    parser.add_argument("--database", type=Path, default=Path("data/notes.sqlite3"))
    commands = parser.add_subparsers(dest="command", required=True)
    sync = commands.add_parser(
        "index", help="Synchronize a directory, including updates and deleted documents"
    )
    sync.add_argument("directory", type=Path)
    search = commands.add_parser(
        "search", help="Rank documents containing all query words"
    )
    search.add_argument("query")
    search.add_argument("--limit", type=int, default=20)
    search.add_argument("--json", action="store_true")
    show = commands.add_parser("show", help="Print the indexed content of a result id")
    show.add_argument("id", type=int)
    commands.add_parser("stats", help="List indexed roots and document counts")
    args = parser.parse_args(argv)
    try:
        index = Index(args.database)
        if args.command == "index":
            result = index.sync(args.directory)
        elif args.command == "stats":
            result = index.stats()
        elif args.command == "show":
            print(index.document(args.id)["content"], end="")
            return 0
        else:
            result = index.search(args.query, args.limit)
            if not args.json:
                if not result:
                    print("No matching documents.")
                for row in result:
                    print(
                        f"{row['id']:>4}  {row['title']}\n      {row['root']}/{row['path']}\n      {row['excerpt']}\n"
                    )
                return 0
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    except (OSError, ValueError, sqlite3.Error) as error:
        print(f"fieldnotes: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
