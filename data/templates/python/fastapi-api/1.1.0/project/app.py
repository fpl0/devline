from pathlib import Path
from threading import Lock
from typing import Annotated

from fastapi import FastAPI, Path as PathParameter, Request, Response
from fastapi.exceptions import RequestValidationError
from fastapi.openapi.docs import get_swagger_ui_html
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, ConfigDict, StringConstraints


class TaskInput(BaseModel):
    model_config = ConfigDict(extra="forbid")
    title: Annotated[str, StringConstraints(strip_whitespace=True, min_length=1, max_length=120)]
    completed: bool = False


class Task(TaskInput):
    model_config = ConfigDict(extra="forbid", frozen=True)
    id: int


class TaskNotFound(Exception):
    pass


class TaskStore:
    def __init__(self):
        self._lock = Lock()
        self._tasks: dict[int, Task] = {}
        self._next_id = 1

    def list(self) -> list[Task]:
        with self._lock:
            return list(self._tasks.values())

    def add(self, data: TaskInput) -> Task:
        with self._lock:
            task = Task(id=self._next_id, **data.model_dump())
            self._tasks[task.id] = task
            self._next_id += 1
            return task

    def get(self, task_id: int) -> Task:
        with self._lock:
            task = self._tasks.get(task_id)
            if task is None:
                raise TaskNotFound
            return task

    def replace(self, task_id: int, data: TaskInput) -> Task:
        with self._lock:
            if task_id not in self._tasks:
                raise TaskNotFound
            task = Task(id=task_id, **data.model_dump())
            self._tasks[task_id] = task
            return task

    def delete(self, task_id: int) -> None:
        with self._lock:
            if self._tasks.pop(task_id, None) is None:
                raise TaskNotFound


TaskId = Annotated[int, PathParameter(gt=0)]
SWAGGER_DIR = Path(__file__).parent / "swagger"


def create_app() -> FastAPI:
    api = FastAPI(title="__PROJECT_NAME__", docs_url=None, redoc_url=None)
    api.mount("/swagger", StaticFiles(directory=SWAGGER_DIR), name="swagger")
    tasks = TaskStore()

    @api.exception_handler(TaskNotFound)
    async def task_not_found(_: Request, _error: TaskNotFound) -> JSONResponse:
        return JSONResponse(
            status_code=404,
            content={"error": {"code": "task_not_found", "message": "That task does not exist."}},
        )

    @api.exception_handler(RequestValidationError)
    async def validation_error(_: Request, error: RequestValidationError) -> JSONResponse:
        details = [
            {"location": item["loc"], "message": item["msg"]}
            for item in error.errors()
        ]
        return JSONResponse(
            status_code=422,
            content={
                "error": {
                    "code": "invalid_input",
                    "message": "Check the request fields.",
                    "details": details,
                }
            },
        )

    @api.get("/healthz")
    def health() -> dict[str, str]:
        return {"status": "ok"}

    @api.get("/tasks")
    def list_tasks() -> list[Task]:
        return tasks.list()

    @api.post("/tasks", status_code=201)
    def add_task(data: TaskInput, response: Response) -> Task:
        task = tasks.add(data)
        response.headers["Location"] = f"/tasks/{task.id}"
        return task

    @api.get("/tasks/{task_id}")
    def show_task(task_id: TaskId) -> Task:
        return tasks.get(task_id)

    @api.put("/tasks/{task_id}")
    def replace_task(task_id: TaskId, data: TaskInput) -> Task:
        return tasks.replace(task_id, data)

    @api.delete("/tasks/{task_id}", status_code=204)
    def delete_task(task_id: TaskId) -> Response:
        tasks.delete(task_id)
        return Response(status_code=204)

    @api.get("/docs", include_in_schema=False)
    def docs():
        return get_swagger_ui_html(
            openapi_url="/openapi.json",
            title=f"{api.title} · Swagger UI",
            swagger_js_url="/swagger/swagger-ui-bundle.js",
            swagger_css_url="/swagger/swagger-ui.css",
            swagger_favicon_url="/swagger/favicon-32x32.png",
            swagger_ui_parameters={"validatorUrl": None},
        )

    return api


app = create_app()
