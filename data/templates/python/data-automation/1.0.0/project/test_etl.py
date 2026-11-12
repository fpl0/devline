import json
from pathlib import Path
import subprocess
import sys

import pytest
from observatory import connect, ingest, render_html, report

FIXTURES = Path(__file__).parent / "fixtures"


def test_ingestion_report_and_restart(tmp_path):
    path = tmp_path / "sales.db"
    with connect(path) as database:
        result = ingest(database, FIXTURES / "sales.csv")
        assert result["accepted"] == 6
        assert ingest(database, FIXTURES / "sales.csv")["already_imported"]
        assert report(database)["total_cents"] == 17593
    with connect(path) as database:
        assert report(database, "2026-10-01", "2026-10-31")["total_cents"] == 3249
        assert report(database)["imports"] == 1


def test_quality_quarantine_and_duplicate_conflict(tmp_path):
    with connect(tmp_path / "sales.db") as database:
        result = ingest(database, FIXTURES / "rejected.csv")
        assert (result["accepted"], result["rejected"], result["duplicates"]) == (
            1,
            3,
            1,
        )
        assert len(result["errors"]) == 3
        assert report(database)["total_cents"] == 2598


def test_wrong_headers_and_empty_files(tmp_path):
    source = tmp_path / "bad.csv"
    with connect(tmp_path / "sales.db") as database:
        source.write_text("wrong,columns\na,b\n")
        with pytest.raises(ValueError):
            ingest(database, source)
        source.write_text("")
        with pytest.raises(ValueError):
            ingest(database, source)
        with pytest.raises(ValueError):
            report(database, "2026-12-01", "2026-01-01")


def test_html_escapes_user_data():
    data = {
        "categories": [
            {
                "category": "<script>alert(1)</script>",
                "orders": 1,
                "units": 1,
                "revenue_cents": 123,
            }
        ],
        "months": [],
        "since": None,
        "until": None,
        "total_cents": 123,
        "imports": 1,
    }
    page = render_html(data)
    assert "<script>" not in page
    assert "&lt;script&gt;" in page and "€1.23" in page


def test_cli_reports_reject_exit_status_and_writes_report(tmp_path):
    base = [sys.executable, "etl.py", "--database", str(tmp_path / "sales.db")]
    result = subprocess.run(
        base
        + [
            "ingest",
            str(FIXTURES / "rejected.csv"),
            "--rejects",
            str(tmp_path / "rejects.json"),
        ],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 2
    assert len(json.loads((tmp_path / "rejects.json").read_text())) == 3
    subprocess.run(
        base + ["report", "--html", str(tmp_path / "report.html")],
        check=True,
        capture_output=True,
    )
    assert "Sales Observatory" in (tmp_path / "report.html").read_text()


def test_duplicate_headers_fail_and_quoted_newlines_are_preserved(tmp_path):
    source = tmp_path / "sales.csv"
    with connect(tmp_path / "sales.db") as database:
        source.write_text(
            "order_id,date,category,quantity,quantity,unit_price_cents\na,2026-01-01,Books,1,9,100\n"
        )
        with pytest.raises(ValueError):
            ingest(database, source)
        source.write_text(
            'order_id,date,category,quantity,unit_price_cents\na,2026-01-01,"Books\nGifts",1,100\n'
        )
        assert ingest(database, source)["accepted"] == 1
        assert report(database)["categories"][0]["category"] == "Books\nGifts"
