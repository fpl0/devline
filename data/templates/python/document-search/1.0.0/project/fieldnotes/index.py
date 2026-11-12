from contextlib import contextmanager
import hashlib
from pathlib import Path
import re
import sqlite3

SUPPORTED = {".md", ".txt", ".rst"}
MAX_BYTES = 2 * 1024 * 1024


class Index:
    def __init__(self, database):
        self.database = Path(database)
        self.database.parent.mkdir(parents=True, exist_ok=True)
        with self.connect() as connection:
            connection.executescript("""
                PRAGMA journal_mode=WAL;
                CREATE TABLE IF NOT EXISTS documents (
                    id INTEGER PRIMARY KEY, root TEXT NOT NULL, path TEXT NOT NULL,
                    title TEXT NOT NULL, content TEXT NOT NULL, digest TEXT NOT NULL,
                    UNIQUE(root,path));
                CREATE VIRTUAL TABLE IF NOT EXISTS search USING fts5(
                    title,content,content='documents',content_rowid='id',tokenize='unicode61');
                CREATE TRIGGER IF NOT EXISTS insert_doc AFTER INSERT ON documents BEGIN
                    INSERT INTO search(rowid,title,content) VALUES(new.id,new.title,new.content);
                END;
                CREATE TRIGGER IF NOT EXISTS delete_doc AFTER DELETE ON documents BEGIN
                    INSERT INTO search(search,rowid,title,content) VALUES('delete',old.id,old.title,old.content);
                END;
                CREATE TRIGGER IF NOT EXISTS update_doc AFTER UPDATE ON documents BEGIN
                    INSERT INTO search(search,rowid,title,content) VALUES('delete',old.id,old.title,old.content);
                    INSERT INTO search(rowid,title,content) VALUES(new.id,new.title,new.content);
                END;
            """)

    @contextmanager
    def connect(self):
        connection = sqlite3.connect(self.database, timeout=10)
        connection.row_factory = sqlite3.Row
        try:
            with connection:
                yield connection
        finally:
            connection.close()

    def sync(self, directory):
        root = Path(directory).resolve(strict=True)
        if not root.is_dir():
            raise ValueError("Index source must be a directory")
        outcome = {
            "added": 0,
            "updated": 0,
            "unchanged": 0,
            "removed": 0,
            "skipped": [],
        }
        with self.connect() as db:
            db.execute("BEGIN IMMEDIATE")
            existing = {
                row["path"]: row
                for row in db.execute(
                    "SELECT id,path,digest FROM documents WHERE root=?", (str(root),)
                )
            }
            seen = set()
            for file in sorted(root.rglob("*")):
                relative = file.relative_to(root).as_posix()
                if (
                    file.is_symlink()
                    or any(part.startswith(".") for part in Path(relative).parts)
                    or file.suffix.lower() not in SUPPORTED
                ):
                    continue
                if not file.is_file():
                    continue
                seen.add(relative)
                try:
                    if file.stat().st_size > MAX_BYTES:
                        raise ValueError("larger than 2 MiB")
                    raw = file.read_bytes()
                    content = raw.decode("utf-8")
                    if "\x00" in content:
                        raise ValueError("contains binary data")
                except (OSError, UnicodeError, ValueError) as error:
                    outcome["skipped"].append({"path": relative, "reason": str(error)})
                    continue
                digest = hashlib.sha256(raw).hexdigest()
                if relative in existing and existing[relative]["digest"] == digest:
                    outcome["unchanged"] += 1
                    continue
                heading = next(
                    (
                        line.strip().lstrip("#").strip()
                        for line in content.splitlines()
                        if line.strip()
                    ),
                    file.stem,
                )
                db.execute(
                    """INSERT INTO documents(root,path,title,content,digest) VALUES(?,?,?,?,?)
                    ON CONFLICT(root,path) DO UPDATE SET title=excluded.title,content=excluded.content,digest=excluded.digest""",
                    (str(root), relative, heading[:200], content, digest),
                )
                outcome["updated" if relative in existing else "added"] += 1
            for missing in existing.keys() - seen:
                db.execute(
                    "DELETE FROM documents WHERE root=? AND path=?",
                    (str(root), missing),
                )
                outcome["removed"] += 1
        return outcome

    def search(self, query, limit=20):
        words = re.findall(r"\w+", query, re.UNICODE)
        if not words or len(words) > 30:
            raise ValueError("Search needs 1–30 words")
        if not 1 <= limit <= 100:
            raise ValueError("Limit must be between 1 and 100")
        expression = " AND ".join('"' + word + '"' for word in words)
        with self.connect() as db:
            return [
                dict(row)
                for row in db.execute(
                    """SELECT documents.id,root,path,documents.title,
                snippet(search,1,'[',']',' … ',24) AS excerpt
                FROM search JOIN documents ON documents.id=search.rowid
                WHERE search MATCH ? ORDER BY bm25(search,5.0,1.0),root,path LIMIT ?""",
                    (expression, limit),
                )
            ]

    def document(self, identity):
        with self.connect() as db:
            row = db.execute(
                "SELECT id,root,path,title,content FROM documents WHERE id=?",
                (identity,),
            ).fetchone()
            if row is None:
                raise ValueError("No indexed document has that id")
            return dict(row)

    def stats(self):
        with self.connect() as db:
            return [
                dict(row)
                for row in db.execute(
                    "SELECT root,count(*) AS documents,sum(length(content)) AS characters FROM documents GROUP BY root ORDER BY root"
                )
            ]
