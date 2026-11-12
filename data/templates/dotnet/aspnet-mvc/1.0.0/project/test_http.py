"""Exercise rendered forms, permissions, CSRF, immutable decisions and persistence."""

import html.parser
import http.cookiejar
import os
import pathlib
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

stack = sys.argv[1]
STARTUP_TIMEOUT_SECONDS = 90.0


class Fields(html.parser.HTMLParser):
    def __init__(self, html):
        super().__init__()
        self.values = {}
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "input" and a.get("name"):
            self.values[a["name"]] = a.get("value", "")


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar())
    )


def call(opener, path, data=None, expected=200, timeout=10):
    request = urllib.request.Request(
        base + path,
        data=urllib.parse.urlencode(data).encode() if data is not None else None,
    )
    try:
        response = opener.open(request, timeout=timeout)
    except urllib.error.HTTPError as error:
        response = error
    body = response.read().decode()
    assert response.status == expected, (path, response.status, body[:800])
    return body, response.url


csrf_name = "_csrf" if stack == "java" else "__RequestVerificationToken"
login_path = "/login" if stack == "java" else "/account/login"


def token(page):
    return Fields(page).values[csrf_name]


def login(username):
    opener = client()
    page, _ = call(opener, login_path)
    call(
        opener,
        login_path,
        {"username": username, "password": "devline-local", csrf_name: token(page)},
    )
    return opener


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

    def start():
        global base, server
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        base = f"http://127.0.0.1:{port}"
        env = os.environ.copy()
        if stack == "java":
            env["PURCHASE_DATABASE"] = "jdbc:h2:file:" + temporary + "/purchases"
            command = [
                str(pathlib.Path(env["JAVA_HOME"]) / "bin/java")
                if env.get("JAVA_HOME")
                else "java",
                "-XX:ActiveProcessorCount=2",
                "-jar",
                "target/task-board.jar",
                f"--server.port={port}",
            ]
            health = "/health"
        else:
            env["ConnectionStrings__Purchases"] = (
                "Data Source=" + temporary + "/purchases.db"
            )
            env["ASPNETCORE_URLS"] = base
            command = [
                env.get("DEVLINE_TOOLCHAIN", env.get("DEVKIT_TOOLCHAIN", "dotnet")),
                "bin/Debug/net10.0/TaskBoard.dll",
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
                call(
                    client(),
                    health,
                    timeout=min(2.0, max(0.001, deadline - time.monotonic())),
                )
                return
            except (OSError, AssertionError):
                time.sleep(min(0.1, max(0, deadline - time.monotonic())))
        log.flush()
        log.seek(0)
        raise AssertionError("Startup failed: " + log.read())

    try:
        start()
        anonymous = client()
        _, url = call(anonymous, "/")
        assert login_path in url
        alice = login("alice")
        page, _ = call(alice, "/")
        csrf = token(page)
        data = {
            "title": "Workshop equipment",
            "reason": "Hands-on learning for the team",
            "amount": "240.50",
        }
        call(alice, "/requests/create", data, 403 if stack == "java" else 400)
        for bad in ["0", "-1", "1.001", "1000000.01"]:
            call(
                alice,
                "/requests/create",
                dict(data, amount=bad, **{csrf_name: csrf}),
                400,
            )
        detail, url = call(alice, "/requests/create", dict(data, **{csrf_name: csrf}))
        resource = urllib.parse.urlparse(url).path
        assert (
            "240.50" in detail
            and "Workshop equipment" in detail
            and "submitted" in detail
        )
        call(
            alice,
            resource + "/decision",
            {
                "version": 1,
                "decision": "approved",
                "note": "Self approval",
                csrf_name: csrf,
            },
            403,
        )
        bob = login("bob")
        detail, _ = call(bob, resource)
        csrf_bob = token(detail)
        call(
            bob,
            resource + "/decision",
            {"version": 1, "decision": "approved", "note": "", csrf_name: csrf_bob},
            400,
        )
        detail, _ = call(
            bob,
            resource + "/decision",
            {
                "version": 1,
                "decision": "approved",
                "note": "Within the team budget",
                csrf_name: csrf_bob,
            },
        )
        assert "approved" in detail and "Within the team budget" in detail
        call(
            bob,
            resource + "/decision",
            {
                "version": 2,
                "decision": "rejected",
                "note": "Changed mind",
                csrf_name: csrf_bob,
            },
            409,
        )
        page, _ = call(bob, "/")
        csrf_bob = token(page)
        own, url = call(
            bob,
            "/requests/create",
            dict(data, title="Bob request", **{csrf_name: csrf_bob}),
        )
        ownpath = urllib.parse.urlparse(url).path
        call(
            bob,
            ownpath + "/decision",
            {
                "version": 1,
                "decision": "approved",
                "note": "Own item",
                csrf_name: csrf_bob,
            },
            403,
        )
        filtered, _ = call(bob, "/?status=approved&q=Workshop")
        assert "Workshop equipment" in filtered and "Bob request" not in filtered
        # Templates escape content instead of interpreting it as markup.
        page, url = call(
            alice,
            "/requests/create",
            dict(data, title="<script>alert(1)</script>", **{csrf_name: csrf}),
        )
        assert "<script>alert(1)</script>" not in page
        stop()
        start()
        bob = login("bob")
        detail, _ = call(bob, resource)
        assert (
            "approved" in detail
            and "Within the team budget" in detail
            and "submitted" in detail
        )
        print(
            "Rendered forms, money validation, roles, CSRF, self-approval guard, immutable decisions, escaping, filtering and restart persistence passed."
        )
    finally:
        stop()
        log.close()
