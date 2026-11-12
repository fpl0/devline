import concurrent.futures
from pathlib import Path
import tempfile
import unittest

from fastapi.testclient import TestClient
from app import create_app


class OrderWorkflowTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.path = Path(self.temp.name) / "orders.db"
        self.client = TestClient(create_app(self.path))
        self.client.post(
            "/products",
            json={
                "sku": "BOOK",
                "name": "Field Notes",
                "price_cents": 1299,
                "stock": 5,
            },
        )
        self.request = {"customer": "Alex", "lines": [{"sku": "BOOK", "quantity": 2}]}

    def tearDown(self):
        self.client.close()
        self.temp.cleanup()

    def place(self, key="first"):
        return self.client.post(
            "/orders", json=self.request, headers={"Idempotency-Key": key}
        )

    def test_full_order_workflow_and_restart(self):
        created = self.place()
        self.assertEqual(created.status_code, 201)
        order = created.json()
        self.assertEqual(order["total_cents"], 2598)
        self.assertEqual(self.client.get("/products").json()[0]["stock"], 3)
        result = self.client.patch(
            f"/orders/{order['id']}", json={"status": "fulfilled"}
        )
        self.assertEqual(result.json()["status"], "fulfilled")
        with TestClient(create_app(self.path)) as restarted:
            self.assertEqual(
                len(restarted.get(f"/orders/{order['id']}").json()["history"]), 2
            )
            self.assertEqual(
                restarted.get("/reports/sales").json()["by_status"][0]["total_cents"],
                2598,
            )
        self.assertEqual(
            self.client.patch(
                f"/orders/{order['id']}", json={"status": "cancelled"}
            ).status_code,
            409,
        )

    def test_idempotency_and_cancel_restore_stock_once(self):
        order = self.place().json()
        self.assertEqual(self.place().status_code, 200)
        self.request["customer"] = "Different"
        self.assertEqual(self.place().status_code, 409)
        for _ in range(2):
            self.assertEqual(
                self.client.patch(
                    f"/orders/{order['id']}", json={"status": "cancelled"}
                ).status_code,
                200,
            )
        self.assertEqual(self.client.get("/products").json()[0]["stock"], 5)

    def test_failed_order_is_atomic(self):
        self.request["lines"].append({"sku": "MISSING", "quantity": 1})
        self.assertEqual(self.place().status_code, 422)
        self.assertEqual(self.client.get("/products").json()[0]["stock"], 5)
        self.assertEqual(self.client.get("/orders").json(), [])

    def test_concurrent_orders_cannot_oversell(self):
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            codes = list(
                pool.map(lambda key: self.place(str(key)).status_code, range(3))
            )
        self.assertEqual(sorted(codes), [201, 201, 409])
        self.assertEqual(self.client.get("/products").json()[0]["stock"], 1)

    def test_validation_unknown_order_and_offline_docs(self):
        self.assertEqual(self.client.get("/orders/missing").status_code, 404)
        self.assertEqual(
            self.client.post(
                "/products",
                json={"sku": "BAD", "name": " ", "price_cents": 1.5, "stock": -1},
            ).status_code,
            422,
        )
        self.assertEqual(self.client.get("/orders?limit=0").status_code, 422)
        self.request["lines"] *= 2
        self.assertEqual(self.place().status_code, 422)
        self.assertIn("/swagger/swagger-ui-bundle.js", self.client.get("/docs").text)


if __name__ == "__main__":
    unittest.main()
