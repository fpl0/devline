import json
import subprocess
import sys
from pathlib import Path

import httpx
import pytest

from etl import download_json, summarize


FIXTURES = Path(__file__).parent / "fixtures"
EXPECTED = [
    {"category": "books", "rows": 2, "total_cents": 2000},
    {"category": "coffee", "rows": 1, "total_cents": 450},
]


@pytest.mark.parametrize("suffix", ["csv", "json"])
def test_local_format(suffix):
    assert summarize(FIXTURES / f"sales.{suffix}") == EXPECTED


def test_optional_http_source_without_network(tmp_path):
    payload = json.loads((FIXTURES / "sales.json").read_text())
    transport = httpx.MockTransport(lambda request: httpx.Response(200, json=payload))
    target = tmp_path / "input.json"
    with httpx.Client(transport=transport) as client:
        download_json(client, "http://127.0.0.1/sales", target)
    assert summarize(target) == EXPECTED


def test_cli_writes_json(tmp_path):
    target = tmp_path / "summary.json"
    subprocess.run(
        [sys.executable, "etl.py", "--input", str(FIXTURES / "sales.csv"), "--output", str(target)],
        check=True,
    )
    assert json.loads(target.read_text()) == EXPECTED
