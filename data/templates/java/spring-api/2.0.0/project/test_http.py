"""Exercise the real HTTP server, concurrent edits and persisted data after restart."""

import concurrent.futures
import json
import os
import pathlib
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

stack = sys.argv[1]
STARTUP_TIMEOUT_SECONDS = 90.0
with tempfile.TemporaryDirectory() as temporary:
    log = open(pathlib.Path(temporary) / "server.log", "w+")
    server = None

    def stop():
        global server
        if server:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=5)
            server = None

    def request(method, path, data=None, expected=200, timeout=10):
        req = urllib.request.Request(
            base + path,
            data=json.dumps(data).encode() if data is not None else None,
            headers={"Content-Type": "application/json"},
            method=method,
        )
        try:
            response = urllib.request.urlopen(req, timeout=timeout)
        except urllib.error.HTTPError as error:
            response = error
        payload = response.read()
        assert response.status == expected, (method, path, response.status, payload)
        return json.loads(payload) if payload else None

    def start():
        global server, base
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        base = f"http://127.0.0.1:{port}"
        env = os.environ.copy()
        if stack == "java":
            env["ISSUE_DATABASE"] = "jdbc:h2:file:" + temporary + "/issues"
            command = [
                str(pathlib.Path(env["JAVA_HOME"]) / "bin/java")
                if env.get("JAVA_HOME")
                else "java",
                "-XX:ActiveProcessorCount=2",
                "-jar",
                "target/task-api.jar",
                f"--server.port={port}",
            ]
            health = "/health"
        else:
            env["ConnectionStrings__Issues"] = "Data Source=" + temporary + "/issues.db"
            env["ASPNETCORE_URLS"] = base
            command = [
                env.get("DEVLINE_TOOLCHAIN", env.get("DEVKIT_TOOLCHAIN", "dotnet")),
                "bin/Debug/net10.0/OfflineApi.dll",
            ]
            health = "/health/ready"
        server = subprocess.Popen(
            command, env=env, stdout=log, stderr=subprocess.STDOUT
        )
        deadline = time.monotonic() + STARTUP_TIMEOUT_SECONDS
        while time.monotonic() < deadline:
            if server.poll() is not None:
                break
            try:
                request(
                    "GET",
                    health,
                    timeout=min(2.0, max(0.001, deadline - time.monotonic())),
                )
                return
            except (OSError, AssertionError):
                time.sleep(min(0.1, max(0, deadline - time.monotonic())))
        log.flush()
        log.seek(0)
        raise AssertionError("Server did not start: " + log.read())

    try:
        start()
        request("POST", "/api/issues", {"title": " ", "priority": "high"}, 400)
        request(
            "POST",
            "/api/issues",
            {"title": "Bad priority", "priority": "critical"},
            400,
        )
        issue = request(
            "POST",
            "/api/issues",
            {
                "title": " Offline indexing ",
                "description": "Fresh cache fails",
                "priority": "high",
            },
            201,
        )
        resource = "/api/issues/" + str(issue["id"])
        assert issue["title"] == "Offline indexing" and issue["version"] == 1
        change = {
            "title": issue["title"],
            "description": issue["description"],
            "priority": "high",
            "status": "resolved",
            "version": 1,
        }
        request("PUT", resource, change, 409)
        change["status"] = "in_progress"
        updated = request("PUT", resource, change)
        assert updated["version"] == 2
        request("PUT", resource, change, 409)
        assert len(request("GET", "/api/issues?q=INDEXING&status=in_progress")) == 1
        request("GET", "/api/issues?limit=1000", expected=400)
        request("GET", "/api/issues/999999", expected=404)
        # Two clients loaded version 2. Exactly one edit may commit.
        change["version"] = 2

        def race(title):
            req = urllib.request.Request(
                base + resource,
                data=json.dumps(dict(change, title=title)).encode(),
                headers={"Content-Type": "application/json"},
                method="PUT",
            )
            try:
                return urllib.request.urlopen(req, timeout=10).status
            except urllib.error.HTTPError as error:
                return error.code

        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as workers:
            assert sorted(workers.map(race, ["First editor", "Second editor"])) == [
                200,
                409,
            ]
        persisted = request("GET", resource)
        assert persisted["version"] == 3
        assert len(request("GET", resource + "/events")) == 3
        stop()
        start()
        assert request("GET", resource) == persisted
        change.update(title=persisted["title"], status="resolved", version=3)
        assert request("PUT", resource, change)["status"] == "resolved"
        change.update(status="in_progress", version=4)
        request("PUT", resource, change, 409)
        change["status"] = "open"
        assert request("PUT", resource, change)["version"] == 5
        print(
            "HTTP validation, workflow, search, concurrent edits, audit history and restart persistence passed."
        )
    finally:
        stop()
        log.close()
