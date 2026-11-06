import unittest

from httpx import ASGITransport, AsyncClient

from app import create_app


class ApiTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.client = AsyncClient(transport=ASGITransport(app=create_app()), base_url="http://testserver")
        self.addAsyncCleanup(self.client.aclose)

    async def test_task_lifecycle(self):
        self.assertEqual((await self.client.get("/healthz")).json(), {"status": "ok"})
        created = await self.client.post("/tasks", json={"title": "  Write a test "})
        self.assertEqual(created.status_code, 201)
        self.assertEqual(created.headers["location"], "/tasks/1")
        original = {"id": 1, "title": "Write a test", "completed": False}
        self.assertEqual(created.json(), original)
        self.assertEqual((await self.client.get("/tasks")).json(), [original])
        self.assertEqual((await self.client.get("/tasks/1")).json(), original)
        replacement = await self.client.put("/tasks/1", json={"title": "Finish it", "completed": True})
        self.assertEqual(replacement.status_code, 200)
        self.assertEqual(replacement.json(), {"id": 1, "title": "Finish it", "completed": True})
        self.assertEqual((await self.client.delete("/tasks/1")).status_code, 204)
        missing = await self.client.get("/tasks/1")
        self.assertEqual(missing.status_code, 404)
        self.assertEqual(missing.json()["error"]["code"], "task_not_found")

    async def test_validation(self):
        for payload in ({"title": " "}, {"title": "x" * 121}, {"title": "Valid", "unknown": 1}):
            response = await self.client.post("/tasks", json=payload)
            self.assertEqual(response.status_code, 422)
            self.assertEqual(response.json()["error"]["code"], "invalid_input")
        self.assertEqual((await self.client.get("/tasks/0")).status_code, 422)

    async def test_local_api_docs(self):
        docs = await self.client.get("/docs")
        self.assertEqual(docs.status_code, 200)
        for asset in ("swagger-ui-bundle.js", "swagger-ui.css", "favicon-32x32.png"):
            route = f"/swagger/{asset}"
            self.assertIn(route, docs.text)
            self.assertEqual((await self.client.get(route)).status_code, 200)
        schema = await self.client.get("/openapi.json")
        self.assertEqual(schema.status_code, 200)
        self.assertIn("/tasks/{task_id}", schema.json()["paths"])


if __name__ == "__main__":
    unittest.main()
