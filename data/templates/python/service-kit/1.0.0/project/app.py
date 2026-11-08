import asyncio
import smtplib
import sqlite3
from typing import Annotated

import httpx
import pika
import psycopg
from botocore.exceptions import BotoCoreError, ClientError
from fastapi import FastAPI, HTTPException, Path, Query, Request
from fastapi.responses import JSONResponse
from nats.errors import Error as NatsError
from pydantic import BaseModel, ConfigDict, StringConstraints
from pydantic_settings import BaseSettings
from structlog import get_logger

import support


class Settings(BaseSettings):
    environment: str = "local"


class TicketInput(BaseModel):
    model_config = ConfigDict(extra="forbid")
    title: Annotated[str, StringConstraints(strip_whitespace=True, min_length=1, max_length=120, pattern=r"^[^\r\n\x00]+$")]
    details: Annotated[str, StringConstraints(strip_whitespace=True, min_length=1, max_length=4_000, pattern=r"^[^\x00]+$")]


TicketID = Annotated[str, Path(pattern=r"^[a-f0-9]{32}$")]
app = FastAPI(title="__PROJECT_NAME__", docs_url=None, redoc_url=None)
logger = get_logger()


async def service_error(_request: Request, error: Exception):
    # Service URLs and exception messages may include generated credentials.
    logger.warning("support_service_unavailable", error=type(error).__name__)
    return JSONResponse({"detail": "Local service unavailable. Check Devline services."}, status_code=503)


for error in (
    support.ServiceUnavailable, psycopg.Error, sqlite3.Error, pika.exceptions.AMQPError,
    smtplib.SMTPException, OSError, httpx.HTTPError, BotoCoreError, ClientError, NatsError,
):
    app.add_exception_handler(error, service_error)


def ticket_or_404(id_):
    ticket = support.read_ticket(id_)
    if ticket is None:
        raise HTTPException(404, "Ticket not found")
    return ticket


@app.get("/")
def index():
    return {
        "app": "Support Desk",
        "create": "POST /tickets with title and details",
        "list": "GET /tickets",
        "schema": "/openapi.json",
    }


@app.get("/health")
def health():
    settings = Settings()
    logger.info("health_checked", environment=settings.environment)
    return {"status": "ok", "environment": settings.environment}


@app.post("/tickets", status_code=201)
def create_ticket(body: TicketInput):
    return support.create_ticket(body.title, body.details)


@app.get("/tickets")
def list_tickets():
    return support.list_tickets()


@app.get("/tickets/{id_}")
def show_ticket(id_: TicketID):
    return ticket_or_404(id_)


@app.post("/tickets/{id_}/notify", status_code=202)
def notify(id_: TicketID):
    support.enqueue_notification(ticket_or_404(id_))
    return {"id": id_, "status": "queued"}


@app.post("/notifications/next")
def deliver_notification():
    ticket = support.deliver_notification()
    if not ticket:
        return {"status": "empty"}
    return {"id": ticket["id"], "status": "sent"}


@app.post("/tickets/{id_}/index", status_code=202)
def index_ticket(id_: TicketID):
    return {"task": support.index_ticket(ticket_or_404(id_))}


@app.get("/search/tasks/{task_id}")
def index_status(task_id: int):
    return {"status": support.index_status(task_id)}


@app.get("/search")
def search_tickets(q: Annotated[str, Query(min_length=1, max_length=120)]):
    return support.search_tickets(q)


@app.post("/tickets/{id_}/export", status_code=201)
def export_ticket(id_: TicketID):
    support.export_ticket(ticket_or_404(id_))
    return {"id": id_, "status": "exported"}


@app.get("/tickets/{id_}/export")
def read_export(id_: TicketID):
    ticket = support.read_export(id_)
    if ticket is None:
        raise HTTPException(404, "Export not found")
    return ticket


@app.post("/tickets/{id_}/activity", status_code=201)
def publish_activity(id_: TicketID):
    return asyncio.run(support.activity(id_, ticket_or_404(id_)))


@app.get("/tickets/{id_}/activity")
def read_activity(id_: TicketID):
    event = asyncio.run(support.activity(id_))
    if event is None:
        raise HTTPException(404, "Activity not found")
    return event
