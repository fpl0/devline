from pathlib import Path

from fastapi import FastAPI
from fastapi.openapi.docs import get_swagger_ui_html
from fastapi.staticfiles import StaticFiles

app = FastAPI(docs_url=None, redoc_url=None)
app.mount("/swagger", StaticFiles(directory=Path(__file__).parent / "swagger"), name="swagger")


@app.get("/", response_model=dict[str, str])
def index():
    return {"status": "offline"}


@app.get("/docs", include_in_schema=False)
def docs():
    return get_swagger_ui_html(
        openapi_url=app.openapi_url,
        title="Offline FastAPI · Swagger UI",
        swagger_js_url="/swagger/swagger-ui-bundle.js",
        swagger_css_url="/swagger/swagger-ui.css",
        swagger_favicon_url="/swagger/favicon-32x32.png",
        swagger_ui_parameters={"validatorUrl": None},
    )
