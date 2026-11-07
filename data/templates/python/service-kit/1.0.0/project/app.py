from fastapi import FastAPI
from pydantic_settings import BaseSettings
from structlog import get_logger


class Settings(BaseSettings):
    environment: str = "local"


app = FastAPI(title="__PROJECT_NAME__", docs_url=None, redoc_url=None)
logger = get_logger()


@app.get("/health")
def health():
    settings = Settings()
    logger.info("health_checked", environment=settings.environment)
    return {"status": "ok", "environment": settings.environment}
