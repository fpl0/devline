"""Order Ledger: a local inventory and order API with transactional SQLite storage."""

import os
from pathlib import Path
from typing import Annotated, Literal

from fastapi import FastAPI, Header, Query, Request, Response
from fastapi.openapi.docs import get_swagger_ui_html
from fastapi.responses import JSONResponse, RedirectResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, ConfigDict, Field, StringConstraints

from ledger import Ledger, LedgerError

Name = Annotated[
    str, StringConstraints(strip_whitespace=True, min_length=1, max_length=120)
]
Sku = Annotated[str, StringConstraints(pattern=r"^[A-Z0-9][A-Z0-9_-]{0,39}$")]


class Input(BaseModel):
    model_config = ConfigDict(extra="forbid")


class Product(Input):
    sku: Sku
    name: Name
    price_cents: int = Field(ge=1, le=100_000_000, strict=True)
    stock: int = Field(ge=0, le=1_000_000, strict=True)


class Line(Input):
    sku: Sku
    quantity: int = Field(ge=1, le=10_000, strict=True)


class Order(Input):
    customer: Name
    lines: list[Line] = Field(min_length=1, max_length=100)


class Transition(Input):
    status: Literal["fulfilled", "cancelled"]


class Restock(Input):
    quantity: int = Field(ge=1, le=1_000_000, strict=True)


def create_app(database: str | Path | None = None) -> FastAPI:
    store = Ledger(database or os.getenv("LEDGER_DATABASE", "data/ledger.sqlite3"))
    api = FastAPI(title="Order Ledger", version="1.0.0", docs_url=None, redoc_url=None)
    api.state.ledger = store
    api.mount(
        "/swagger",
        StaticFiles(directory=Path(__file__).parent / "swagger"),
        name="swagger",
    )

    @api.exception_handler(LedgerError)
    async def domain_error(_request: Request, error: LedgerError):
        return JSONResponse(
            status_code=error.status,
            content={"error": error.code, "detail": str(error)},
        )

    @api.get("/", include_in_schema=False)
    def index():
        return RedirectResponse("/docs")

    @api.get("/healthz")
    def health():
        store.products()
        return {"status": "ok"}

    @api.get("/products")
    def products():
        return store.products()

    @api.post("/products", status_code=201)
    def product(data: Product):
        return store.add_product(data.model_dump())

    @api.post("/products/{sku}/stock")
    def restock(sku: str, data: Restock):
        return store.restock(sku, data.quantity)

    @api.post("/orders", status_code=201)
    def order(
        data: Order,
        response: Response,
        idempotency_key: Annotated[str, Header(min_length=1, max_length=100)],
    ):
        result, created = store.place(data.model_dump(), idempotency_key)
        response.status_code = 201 if created else 200
        response.headers["Location"] = f"/orders/{result['id']}"
        return result

    @api.get("/orders")
    def orders(
        status: Literal["placed", "fulfilled", "cancelled"] | None = None,
        limit: int = Query(50, ge=1, le=200),
        offset: int = Query(0, ge=0),
    ):
        return store.orders(status, limit, offset)

    @api.get("/orders/{order_id}")
    def show(order_id: str):
        return store.order(order_id)

    @api.patch("/orders/{order_id}")
    def transition(order_id: str, data: Transition):
        return store.transition(order_id, data.status)

    @api.get("/reports/sales")
    def report():
        return store.report()

    @api.get("/docs", include_in_schema=False)
    def docs():
        return get_swagger_ui_html(
            openapi_url="/openapi.json",
            title="Order Ledger · API",
            swagger_js_url="/swagger/swagger-ui-bundle.js",
            swagger_css_url="/swagger/swagger-ui.css",
            swagger_favicon_url="/swagger/favicon-32x32.png",
            swagger_ui_parameters={"validatorUrl": None},
        )

    return api


app = create_app()
