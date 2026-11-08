import asyncio
import sqlite3
from contextlib import closing

import boto3
import httpx
import jwt
import nats
import pika
import psycopg
import pymysql
import redis
from botocore.stub import Stubber
from cryptography.hazmat.primitives.asymmetric import rsa
from structlog.testing import capture_logs

from app import app


def test_health_configuration_and_logging(monkeypatch):
    monkeypatch.setenv("ENVIRONMENT", "offline-test")
    async def request():
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app), base_url="http://127.0.0.1",
        ) as client:
            return await client.get("/health")

    with capture_logs() as events:
        response = asyncio.run(request())
    assert response.json() == {"status": "ok", "environment": "offline-test"}
    assert events[-1]["event"] == "health_checked"


def test_database_clients_and_sqlite():
    assert psycopg.pq.__impl__ == "binary"
    settings = psycopg.conninfo.conninfo_to_dict(
        psycopg.conninfo.make_conninfo(host="127.0.0.1", dbname="devline")
    )
    assert settings["dbname"] == "devline"
    mysql = pymysql.connect(host="127.0.0.1", user="devline", defer_connect=True)
    assert not mysql.open
    with sqlite3.connect(":memory:") as db:
        db.execute("CREATE TABLE local_items (name TEXT)")
        db.execute("INSERT INTO local_items VALUES (?)", ("offline",))
        assert db.execute("SELECT name FROM local_items").fetchone() == ("offline",)


def test_queue_and_cache_clients():
    with redis.Redis(host="127.0.0.1", port=6379) as cache:
        assert cache.connection_pool.connection_kwargs["host"] == "127.0.0.1"
    broker = pika.ConnectionParameters(host="127.0.0.1", virtual_host="devline")
    assert broker.virtual_host == "devline"
    assert not nats.NATS().is_connected


def test_object_store_http_and_rsa_jwt():
    with closing(boto3.client(
        "s3", endpoint_url="http://127.0.0.1:8333", region_name="us-east-1",
        aws_access_key_id="test", aws_secret_access_key="test",
    )) as s3, Stubber(s3) as responses:
        responses.add_response("list_buckets", {"Buckets": [{"Name": "devline"}]})
        assert s3.list_buckets()["Buckets"][0]["Name"] == "devline"
    with httpx.Client(transport=httpx.MockTransport(lambda request: httpx.Response(200, json={"ok": True}))) as http:
        assert http.get("http://127.0.0.1/health").json() == {"ok": True}
    private_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    token = jwt.encode({"sub": "local-user"}, private_key, algorithm="RS256")
    assert jwt.decode(token, private_key.public_key(), algorithms=["RS256"])["sub"] == "local-user"


def test_ticket_workflow_with_sqlite(monkeypatch, tmp_path):
    monkeypatch.delenv("DEVLINE_POSTGRES_URL", raising=False)
    monkeypatch.delenv("DEVLINE_REDIS_URL", raising=False)
    monkeypatch.delenv("DEVLINE_MEILISEARCH_URL", raising=False)
    monkeypatch.setenv("SUPPORT_SQLITE_PATH", str(tmp_path / "support.sqlite3"))

    async def journey():
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app), base_url="http://127.0.0.1",
        ) as client:
            assert (await client.post("/tickets", json={"title": " ", "details": "Something broke"})).status_code == 422
            created = await client.post("/tickets", json={"title": " Printer ", "details": "Paper jam"})
            assert created.status_code == 201
            ticket = created.json()
            assert ticket["title"] == "Printer"
            assert (await client.get("/tickets")).json() == [ticket]
            assert (await client.get(f"/tickets/{ticket['id']}")).json() == ticket
            assert (await client.get("/tickets/" + "0" * 32)).status_code == 404
            assert (await client.get("/search", params={"q": "Printer"})).status_code == 503

    asyncio.run(journey())
