from fastapi.testclient import TestClient

from app import app

client = TestClient(app)
response = client.get("/")
assert response.status_code == 200 and response.json() == {"status": "offline"}
print("FastAPI test passed")

docs = client.get("/docs")
assert docs.status_code == 200
for asset in ("swagger-ui-bundle.js", "swagger-ui.css", "favicon-32x32.png"):
    assert f"/swagger/{asset}" in docs.text
    assert client.get(f"/swagger/{asset}").status_code == 200
assert client.get("/openapi.json").status_code == 200
print("Local Swagger UI assets passed")
