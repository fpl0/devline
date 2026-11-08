"""Local storage and service operations for the support desk."""

import json
import os
import smtplib
import sqlite3
from contextlib import closing, contextmanager
from email.message import EmailMessage
from pathlib import Path
from threading import Lock
from uuid import uuid4

import boto3
import httpx
import nats
import pika
import psycopg
import redis
from botocore.config import Config
from botocore.exceptions import ClientError
from nats.js.errors import NotFoundError

QUEUE = "support-notifications"
STREAM = "SUPPORT_ACTIVITY"
INDEX = "support-tickets"
BUCKET = "devline"
SCHEMA_LOCK = Lock()


class ServiceUnavailable(Exception):
    pass


def setting(name):
    value = os.environ.get(f"DEVLINE_{name}")
    if not value:
        raise ServiceUnavailable
    return value


def encode(ticket):
    return json.dumps(ticket, ensure_ascii=False)


@contextmanager
def database():
    url = os.environ.get("DEVLINE_POSTGRES_URL")
    if url:
        connection = psycopg.connect(url, connect_timeout=3, options="-c statement_timeout=3000")
        placeholder = "%s"
    else:
        path = Path(os.environ.get("SUPPORT_SQLITE_PATH", "storage/support.sqlite3"))
        path.parent.mkdir(parents=True, exist_ok=True)
        connection = sqlite3.connect(path, timeout=3)
        placeholder = "?"
    with closing(connection), connection:
        with SCHEMA_LOCK:
            connection.execute(
                "CREATE TABLE IF NOT EXISTS support_tickets ("
                "id TEXT PRIMARY KEY, title TEXT NOT NULL, details TEXT NOT NULL)"
            )
            connection.commit()
        yield connection, placeholder


def create_ticket(title, details):
    ticket = {"id": uuid4().hex, "title": title, "details": details}
    with database() as (db, arg):
        db.execute(f"INSERT INTO support_tickets VALUES ({arg}, {arg}, {arg})", tuple(ticket.values()))
    return ticket


def list_tickets():
    with database() as (db, _):
        rows = db.execute("SELECT id, title, details FROM support_tickets ORDER BY id LIMIT 100").fetchall()
    return [dict(id=id_, title=title, details=details) for id_, title, details in rows]


def read_ticket(id_):
    url = os.environ.get("DEVLINE_REDIS_URL")
    if url:
        with redis.Redis.from_url(url, socket_connect_timeout=2, socket_timeout=2) as cache:
            try:
                result = cache.get(f"support:ticket:{id_}")
                if result:
                    return json.loads(result)
            except (redis.RedisError, ValueError):
                pass
            ticket = read_database(id_)
            if ticket:
                try:
                    cache.setex(f"support:ticket:{id_}", 600, encode(ticket))
                except redis.RedisError:
                    pass
            return ticket
    return read_database(id_)


def read_database(id_):
    with database() as (db, arg):
        row = db.execute(
            f"SELECT id, title, details FROM support_tickets WHERE id = {arg}", (id_,)
        ).fetchone()
    return dict(id=row[0], title=row[1], details=row[2]) if row else None


@contextmanager
def queue_channel():
    config = pika.ConnectionParameters(
        host="127.0.0.1",
        port=int(setting("RABBITMQ_AMQP_PORT")),
        virtual_host="devline",
        credentials=pika.PlainCredentials("devline", setting("RABBITMQ_PASSWORD")),
        socket_timeout=3,
        stack_timeout=5,
        blocked_connection_timeout=3,
    )
    connection = pika.BlockingConnection(config)
    try:
        channel = connection.channel()
        channel.queue_declare(queue=QUEUE, durable=True)
        yield channel
    finally:
        connection.close()


def enqueue_notification(ticket):
    with queue_channel() as channel:
        channel.confirm_delivery()
        channel.basic_publish(
            exchange="", routing_key=QUEUE, body=encode(ticket),
            properties=pika.BasicProperties(delivery_mode=2), mandatory=True,
        )


def deliver_notification():
    port = int(setting("MAILPIT_SMTP_PORT"))
    with queue_channel() as channel:
        delivery, _, payload = channel.basic_get(queue=QUEUE)
        if not delivery:
            return None
        ticket = json.loads(payload)
        message = EmailMessage()
        message["From"] = "support@example.test"
        message["To"] = "triage@example.test"
        message["Subject"] = f"Support ticket {ticket['id']}"
        message.set_content(f"{ticket['title']}\n\n{ticket['details']}")
        # Closing the connection on an SMTP failure returns the unacknowledged job to RabbitMQ.
        with smtplib.SMTP("127.0.0.1", port, timeout=4) as smtp:
            smtp.send_message(message)
        channel.basic_ack(delivery.delivery_tag)
        return ticket


@contextmanager
def search_client():
    with httpx.Client(
        base_url=setting("MEILISEARCH_URL"),
        headers={"Authorization": f"Bearer {setting('MEILISEARCH_PASSWORD')}"},
        trust_env=False,
        timeout=4,
    ) as client:
        yield client


def index_ticket(ticket):
    with search_client() as client:
        response = client.post(f"/indexes/{INDEX}/documents", params={"primaryKey": "id"}, json=[ticket])
        return response.raise_for_status().json()["taskUid"]


def index_status(task_id):
    with search_client() as client:
        return client.get(f"/tasks/{task_id}").raise_for_status().json()["status"]


def search_tickets(query):
    with search_client() as client:
        response = client.post(f"/indexes/{INDEX}/search", json={"q": query, "limit": 20})
        if response.status_code == 404:
            return []
        return response.raise_for_status().json()["hits"]


@contextmanager
def object_store():
    with closing(boto3.client(
        "s3", endpoint_url=setting("SEAWEEDFS_URL"), region_name="us-east-1",
        aws_access_key_id="devline", aws_secret_access_key=setting("SEAWEEDFS_PASSWORD"),
        config=Config(connect_timeout=3, read_timeout=3, retries={"total_max_attempts": 1},
                      s3={"addressing_style": "path"}, proxies={}),
    )) as client:
        yield client


def export_ticket(ticket):
    with object_store() as s3:
        s3.put_object(Bucket=BUCKET, Key=f"support/{ticket['id']}.json",
                      Body=encode(ticket).encode(), ContentType="application/json")


def read_export(id_):
    with object_store() as s3:
        try:
            result = s3.get_object(Bucket=BUCKET, Key=f"support/{id_}.json")
        except ClientError as error:
            if error.response["ResponseMetadata"]["HTTPStatusCode"] == 404:
                return None
            raise
        with closing(result["Body"]) as body:
            return json.load(body)


async def activity(id_, ticket=None):
    connection = await nats.connect(
        setting("NATS_URL"), connect_timeout=3, allow_reconnect=False,
    )
    try:
        stream = connection.jetstream(timeout=3)
        subject = f"support.ticket.{id_}"
        if ticket is not None:
            # Identical declarations are idempotent, including after a restart.
            await stream.add_stream(name=STREAM, subjects=["support.ticket.*"], max_msgs=1_000)
            event = {"id": id_, "event": "reviewed", "title": ticket["title"]}
            await stream.publish(subject, encode(event).encode())
            return event
        try:
            result = await stream.get_last_msg(STREAM, subject)
        except NotFoundError:
            return None
        return json.loads(result.data)
    finally:
        await connection.close()
