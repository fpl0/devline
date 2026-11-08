#!/usr/bin/env python3
"""Exercise the bundled Java and .NET demos through Devline on macOS.

Build with `cargo build --release --locked --manifest-path core/Cargo.toml`,
install Maven/JDK and .NET, and prepare the four MVC service images in a
local Devline library before running this script.
The test uses temporary projects, a temporary library and disposable volumes.
"""

import argparse
from contextlib import ExitStack
from http.cookiejar import CookieJar, DefaultCookiePolicy
from html import unescape
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.parse import parse_qs, urlencode, urljoin, urlsplit
from urllib.request import (
    HTTPCookieProcessor,
    HTTPRedirectHandler,
    ProxyHandler,
    Request,
    build_opener,
)

ROOT = Path(__file__).resolve().parents[1]
STARTERS = {
    "java": {"cli": "json-tool", "api": "spring-api", "mvc": "spring-mvc"},
    "dotnet": {"cli": "json-cli", "api": "aspnet-api", "mvc": "aspnet-mvc"},
}
SERVICES = {"keycloak", "postgres", "rabbitmq", "redis"}


def run(arguments, *, cwd=None, env=None, input=None, check=True):
    result = subprocess.run(
        arguments, cwd=cwd, env=env, input=input, text=True,
        capture_output=True, timeout=600,
    )
    if check and result.returncode:
        raise RuntimeError(f"{' '.join(map(str, arguments))} failed:\n{result.stdout}{result.stderr}")
    return result.stdout if check else result


def need(condition, message):
    if not condition:
        raise AssertionError(message)


def port_options(catalog):
    with ExitStack() as reservations:
        # Keep every reservation open until all ports have been selected.
        options = []
        for service in catalog["services"]:
            if service["id"] not in SERVICES:
                continue
            for port in service["ports"]:
                current = reservations.enter_context(socket.socket())
                current.bind(("127.0.0.1", 0))
                options += ["--port", f"{service['id']}:{port['id']}={current.getsockname()[1]}"]
        return options


class Redirects(HTTPRedirectHandler):
    def __init__(self, origins, history, follow):
        self.origins = origins
        self.history = history
        self.follow = follow

    def redirect_request(self, request, response, code, message, headers, url):
        ensure_local(url, self.origins)
        self.history.append(url)
        if self.follow:
            return super().redirect_request(request, response, code, message, headers, url)
        return None


def origin(url):
    parts = urlsplit(url)
    return parts.scheme, parts.netloc


def ensure_local(url, origins):
    need(origin(url) in origins, "The app redirected the test outside its loopback origins")


class Browser:
    def __init__(self, *bases):
        self.origins = {origin(base) for base in bases}
        for scheme, authority in self.origins:
            need(scheme == "http" and urlsplit(f"http://{authority}").hostname == "127.0.0.1",
                 "The test requires HTTP on the IPv4 loopback address")
        # Browser cookie handling permits Secure cookies on trusted loopback HTTP.
        cookies = HTTPCookieProcessor(CookieJar(DefaultCookiePolicy(secure_protocols=("http",))))
        self.redirects = []
        self.openers = {
            follow: build_opener(ProxyHandler({}), cookies, Redirects(self.origins, self.redirects, follow))
            for follow in (True, False)
        }

    def request(self, url, *, method="GET", payload=None, form=None, follow=True, token=None):
        ensure_local(url, self.origins)
        headers = {}
        if token:
            headers["Authorization"] = f"Bearer {token}"
        body = None
        if payload is not None:
            body = json.dumps(payload).encode()
            headers["Content-Type"] = "application/json"
        if form is not None:
            body = urlencode(form).encode()
            headers["Content-Type"] = "application/x-www-form-urlencoded"
        request = Request(url, data=body, headers=headers, method=method)
        try:
            response = self.openers[follow].open(request, timeout=12)
        except HTTPError as error:
            response = error
        with response:
            return response.status, response.headers, response.read().decode(), response.url


class Forms(HTMLParser):
    def __init__(self, document):
        super().__init__()
        self.forms = []
        self.assets = []
        self.current = None
        self.feed(document)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag == "form":
            self.current = (attrs.get("action", ""), {})
            self.forms.append(self.current)
        elif tag == "input" and self.current and "name" in attrs:
            self.current[1][attrs["name"]] = attrs.get("value", "")
        elif tag == "link" and "stylesheet" in attrs.get("rel", "").split():
            self.assets.append(attrs["href"])
        elif tag in ("script", "img") and "src" in attrs:
            self.assets.append(attrs["src"])

    def handle_endtag(self, tag):
        if tag == "form":
            self.current = None

    def matching(self, *fields, target=None):
        return next(((action, inputs) for action, inputs in self.forms
                     if (target is None or urlsplit(action).path == target)
                     and all(field in inputs for field in fields)), None)


def sign_in(base, keycloak, username, password):
    browser = Browser(base, keycloak)
    board = base + "/tasks"
    status, _, page, url = browser.request(board)
    need(browser.redirects and status == 200 and origin(url) == origin(keycloak),
         f"Expected the Keycloak login page, got {describe_response(status, page, url)}")
    authorization = next((parse_qs(urlsplit(redirect).query) for redirect in browser.redirects
                          if origin(redirect) == origin(keycloak)
                          and urlsplit(redirect).path.endswith("/protocol/openid-connect/auth")), {})
    direct = authorization.get("response_type") == ["code"] and authorization.get("code_challenge_method") == ["S256"]
    pushed = authorization.get("request_uri", [""])[0].startswith("urn:ietf:params:oauth:request_uri:")
    need((direct and authorization.get("code_challenge")) or pushed,
         "Sign-in must send S256 PKCE parameters or a pushed authorization request")
    login = Forms(page).matching("username", "password")
    need(login, "Keycloak did not render a username/password form")
    action, inputs = login
    target = urljoin(url, action)
    ensure_local(target, {origin(keycloak)})
    status, _, page, url = browser.request(
        target, method="POST", form={**inputs, "username": username, "password": password},
    )
    callback = Forms(page).matching("code", "state") if origin(url) == origin(keycloak) else None
    if callback:
        action, inputs = callback
        target = urljoin(url, action)
        ensure_local(target, {origin(base)})
        status, _, page, url = browser.request(target, method="POST", form=inputs)
    need(status == 200 and origin(url) == origin(base),
         f"Keycloak login did not return to the task board ({describe_response(status, page, url)})")
    return browser, page, authorization["client_id"][0]


def describe_response(status, page, url):
    message = re.search(r'<p class="instruction">([^<]+)</p>', page)
    detail = f" ({unescape(message[1]).strip()})" if message else ""
    return f"HTTP {status} at {urlsplit(url).path}{detail}"


def verify_keycloak_policy(keycloak, password, stack, client_id):
    browser = Browser(keycloak)
    status, _, response, _ = browser.request(
        keycloak + "/realms/master/protocol/openid-connect/token", method="POST", form={
            "client_id": "admin-cli", "username": "devline", "password": password, "grant_type": "password",
        },
    )
    need(status == 200, "Could not inspect the disposable Keycloak realm")
    token = json.loads(response)["access_token"]
    realm = "java" if stack == "java" else "dotnet"
    query = urlencode({"clientId": client_id})
    status, _, response, _ = browser.request(keycloak + f"/admin/realms/devline-{realm}/clients?{query}", token=token)
    need(status == 200, "Could not inspect the demo's Keycloak client")
    client = next((client for client in json.loads(response) if client["clientId"] == client_id), {})
    need(client.get("attributes", {}).get("pkce.code.challenge.method") == "S256"
         and client.get("standardFlowEnabled") and not client.get("directAccessGrantsEnabled"),
         "Keycloak should require S256 for the demo client and disable direct password grants")


def verify_assets(browser, page, url):
    for asset in Forms(page).assets:
        target = urljoin(url, asset)
        ensure_local(target, {origin(url)})
        status, headers, source, _ = browser.request(target)
        need(status == 200, "A rendered asset was unavailable offline")
        if headers.get_content_type() == "text/css":
            need(not re.search(r"(?i)@import\b|url\(\s*[\"']?(?:https?:)?//", source),
                 "The stylesheet declares an external asset or import")


def task_row(page, title):
    class Rows(HTMLParser):
        def __init__(self):
            super().__init__()
            self.active = None
            self.depth = 0
            self.rows = {}

        def handle_starttag(self, tag, attributes):
            attrs = dict(attributes)
            if "data-task-id" in attrs:
                self.active = (tag, attrs["data-task-id"])
                self.rows[self.active[1]] = []
                self.depth = 1
            elif self.active and self.active[0] == tag:
                self.depth += 1

        def handle_data(self, data):
            if self.active:
                self.rows[self.active[1]].append(data.strip())

        def handle_endtag(self, tag):
            if self.active and self.active[0] == tag:
                self.depth -= 1
                if not self.depth:
                    self.active = None

    parser = Rows()
    parser.feed(page)
    return next(((task, " ".join(parts)) for task, parts in parser.rows.items() if title in parts), None)


def verify_api(base):
    browser = Browser(base)
    status, _, body, _ = browser.request(base + "/")
    need(status == 200 and json.loads(body)["tasks"] == "/api/tasks", "Open App must show the REST API index")
    collection = base + "/api/tasks"
    status, _, body, _ = browser.request(collection)
    need(status == 200 and isinstance(json.loads(body), list), "Task list must return a JSON array")
    status, _, _, _ = browser.request(collection, method="POST", payload={"title": "  "})
    need(status == 400, "The REST API must reject a blank title")
    status, headers, body, _ = browser.request(collection, method="POST", payload={"title": "Read the docs"})
    need(status == 201 and headers.get("Location"), "Task creation must return 201 and a Location")
    task = urljoin(collection, headers["Location"])
    need(json.loads(body)["title"] == "Read the docs", "Task creation returned the wrong title")
    status, _, body, _ = browser.request(task)
    need(status == 200 and json.loads(body)["title"] == "Read the docs", "Could not read the new task")
    status, _, _, _ = browser.request(task, method="PUT", payload={"title": "Try the demo", "completed": True})
    need(status in (200, 204), "Task update failed")
    status, _, body, _ = browser.request(task)
    need(status == 200 and json.loads(body)["completed"] and json.loads(body)["title"] == "Try the demo",
         "The updated task did not persist")
    status, _, _, _ = browser.request(task, method="DELETE")
    need(status == 204 and browser.request(task)[0] == 404, "Deleted task should return 404")


def wait_for_app(project, env):
    run(["./devline.sh", "start"], cwd=project, env=env)
    deadline = time.monotonic() + 150
    while time.monotonic() < deadline:
        result = run(["./devline.sh", "status", "--json"], cwd=project, env=env)
        status = json.loads(result)["data"]
        if status["state"] == "running":
            return status["url"]
        if status["state"] == "failed":
            break
        time.sleep(1)
    raise RuntimeError("The app did not become ready\n" + run(["./devline.sh", "logs"], cwd=project, env=env))


def verify_cli(project, env, stack):
    run(["./devline.sh", "run"], cwd=project, env=env)
    command = ["./devline.sh", "exec", "./.devline/scripts/run.sh"]
    args = ["select", "/items/0", "-"] if stack == "java" else ["get", "/items/0", "-"]
    output = run(command + args, cwd=project, env=env, input='{"items":["devline"]}')
    need(json.loads(output) == "devline", "CLI selection returned an unexpected value")
    invalid = run(command + args, cwd=project, env=env, input="invalid JSON", check=False)
    need(invalid.returncode != 0, "Malformed input should produce a nonzero exit code")


def verify_network_boundary(project, env):
    probe = """\
import errno
import socket

with socket.socket() as connection:
    connection.settimeout(2)
    result = connection.connect_ex(("203.0.113.1", 443))
    if result not in (errno.EACCES, errno.EPERM):
        raise SystemExit(f"Expected OS sandbox to deny external TCP, got errno {result}")
"""
    run(["./devline.sh", "exec", "python3", "-c", probe], cwd=project, env=env)


def verify_service_images(cli, project, env, releases):
    status = run([cli, "--offline", "--json", "infra", "status"], env=env)
    archives = {f"{image['id']}@{image['version']}": image for image in json.loads(status)["data"]["images"]}
    stack = run([cli, "--offline", "--json", "infra", "status", "--project", project], env=env)
    stack = json.loads(stack)["data"]
    need(stack["healthy"], "The isolated service stack was not healthy")
    compose = json.loads((project / ".devline/services/compose.yaml").read_text())
    for reference in releases:
        service = reference.split("@")[0]
        archive = archives[reference]
        need(archive["ready_offline"] and compose["services"][service]["pull_policy"] == "never",
             f"{reference} must use a verified archive without pulling")
        container = next(entry["ID"] for entry in stack["containers"] if entry["Names"].endswith(f"-{service}-1"))
        image, ports, dns = map(json.loads, run([
            "docker", "inspect", "--format", "{{json .Image}}\n{{json .HostConfig.PortBindings}}\n{{json .HostConfig.Dns}}",
            container,
        ]).splitlines())
        need(image == archive["receipt"]["image_id"], f"{reference} did not run the archived image")
        need(dns == ["127.0.0.1"] and all(binding["HostIp"] == "127.0.0.1"
                                          for bindings in ports.values() for binding in bindings),
             f"{reference} must use local DNS and expose only loopback ports")
    network = run(["docker", "network", "inspect", "--format", "{{json .Options}}", stack["project_name"] + "_default"])
    need(json.loads(network)["com.docker.network.bridge.enable_ip_masquerade"] == "false",
         "The disposable stack network allows outbound NAT")


def redis_command(cli, project, env, *args):
    result = run([cli, "--offline", "--json", "infra", "status", "--project", project], env=env)
    containers = json.loads(result)["data"]["containers"]
    redis = next(container["ID"] for container in containers if container["Names"].endswith("-redis-1"))
    return run(["docker", "exec", redis, "redis-cli", "--raw", *args])


def redis_hits(cli, project, env):
    info = redis_command(cli, project, env, "INFO", "stats")
    return int(next(line.split(":", 1)[1] for line in info.splitlines() if line.startswith("keyspace_hits:")))


def verify_mvc(cli, project, env, stack, base):
    result = run([cli, "--offline", "--json", "infra", "credentials", project], env=env)
    connections = json.loads(result)["data"]["environment"]
    keycloak = connections["DEVLINE_KEYCLOAK_URL"]
    user = ("demo", "demo-password")
    browser, page, client_id = sign_in(base, keycloak, *user)
    verify_keycloak_policy(keycloak, connections["DEVLINE_KEYCLOAK_PASSWORD"], stack, client_id)
    board = base + "/tasks"
    verify_assets(browser, page, board)
    status, _, home_page, url = browser.request(base + "/")
    need(status == 200, "The app landing page was unavailable")
    verify_assets(browser, home_page, url)
    form = Forms(page).matching("title")
    need(form, "The signed-in task board did not render a create form")
    action, inputs = form
    target = urljoin(board, action)
    title = "Verify local integrations"
    status, _, _, _ = browser.request(target, method="POST", form={"title": title}, follow=False)
    need(status in (400, 403), "The task board accepted a write without an antiforgery token")
    status, _, _, _ = browser.request(target, method="POST", form={**inputs, "title": title})
    need(status == 200, "Task creation failed after sign-in")
    status, _, page, _ = browser.request(board)
    row = task_row(page, title)
    need(status == 200 and row, "The saved task did not appear after invalidating the cached task list")
    task_id, _ = row

    def mutate(client, document, action, *, follow=True):
        path = f"/tasks/{task_id}/{action}"
        form = Forms(document).matching(target=path)
        need(form, f"The task did not render its {action} form")
        return client.request(base + path, method="POST", form=form[1], follow=follow)

    need(mutate(browser, page, "complete")[0] == 200, "Could not complete the task")

    delivery = "Recorded" if stack == "java" else "Delivered"
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        status, _, page, _ = browser.request(board)
        row = task_row(page, title)
        if status == 200 and row and delivery in row[1]:
            break
        time.sleep(1)
    else:
        raise AssertionError("The task board did not show the saved task and RabbitMQ delivery")

    previous = redis_hits(cli, project, env)
    status, headers, page, _ = browser.request(board)
    need(status == 200 and headers.get("X-Task-Source") == "redis" and task_row(page, title),
         "The cached board did not show the new task")
    need(redis_hits(cli, project, env) > previous, "Reading the task board did not hit Redis")
    reader, other_page, _ = sign_in(base, keycloak, "reader", "reader-password")
    need(task_row(other_page, title) is None, "A different account could see the original user's task")
    reader_form = Forms(other_page).matching("title")
    need(reader_form, "The second user could not get an antiforgery token")
    status, _, _, _ = reader.request(base + f"/tasks/{task_id}/delete", method="POST", form=reader_form[1], follow=False)
    need(status in (302, 303, 403, 404), "Cross-account delete did not return a denial or no-op redirect")

    run(["./devline.sh", "stop"], cwd=project, env=env)
    # The stack belongs to this temporary project. The next app must load from Postgres.
    need(redis_command(cli, project, env, "FLUSHDB").strip() == "OK", "Could not clear the disposable Redis cache")
    base = wait_for_app(project, env)
    browser, page, _ = sign_in(base, keycloak, *user)
    row = task_row(page, title)
    need(row and row[0] == task_id and delivery in row[1], "The task and delivery status did not survive restart")
    status, _, page, _ = mutate(browser, page, "delete")
    need(status == 200 and task_row(page, title) is None, "Deleting the task did not invalidate its cache entry")
    logout_path = "/logout" if stack == "java" else "/account/logout"
    logout = Forms(page).matching(target=logout_path)
    need(logout, "The signed-in task board did not render a sign-out form")
    status, _, _, _ = browser.request(base + logout_path, method="POST", form=logout[1])
    need(status == 200 and 300 <= browser.request(board, follow=False)[0] < 400,
         "Sign-out must clear the application session")


def verify(cli, catalog, project, env, stack, kind):
    template = f"{stack}/{STARTERS[stack][kind]}"
    options = port_options(catalog) if kind == "mvc" else []
    result = run([cli, "--offline", "--json", "new", template, project, *options], env=env)
    generated = json.loads(result)["data"]
    services = {value.split("@")[0] for value in generated["services"]}
    need(generated["ready_offline"] and services == (SERVICES if kind == "mvc" else set()),
         f"Offline generation selected the wrong services for {template}")
    try:
        verify_network_boundary(project, env)
        if kind == "cli":
            verify_cli(project, env, stack)
        else:
            # Avoid collisions with other apps using the same template's default port.
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0))
                config_file = project / ".devline/config.json"
                config = json.loads(config_file.read_text())
                config["run"]["url"] = f"http://127.0.0.1:{reservation.getsockname()[1]}"
                config_file.write_text(json.dumps(config))
            base = wait_for_app(project, env)
            if kind == "api":
                verify_api(base)
            else:
                verify_service_images(cli, project, env, generated["services"])
                verify_mvc(cli, project, env, stack, base)
    finally:
        if kind != "cli":
            run(["./devline.sh", "stop"], cwd=project, env=env)
        if kind == "mvc":
            run([cli, "--offline", "infra", "stop", project, "--delete-volumes"], env=env)
    print(f"PASS {template}@{generated['version']}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=ROOT / "core/target/release/devline")
    parser.add_argument("--vault", type=Path, help="Devline library containing the four prepared image archives")
    parser.add_argument("--stack", choices=STARTERS, action="append")
    parser.add_argument("--kind", choices=("cli", "api", "mvc"), action="append")
    args = parser.parse_args()
    stacks = args.stack or STARTERS
    kinds = args.kind or ("cli", "api", "mvc")
    support = Path.home() / "Library/Application Support"
    vault = args.vault or (support / "Devline" if (support / "Devline").exists() else support / "DevKit")
    catalog = json.loads((ROOT / "data/infrastructure.json").read_text())
    failures = []
    with tempfile.TemporaryDirectory(prefix="devline java dotnet ") as temporary:
        root = Path(temporary).resolve()
        home = root / "library"
        home.mkdir()
        if "mvc" in kinds:
            run(["/bin/cp", "-cR", vault / "infrastructure", home / "infrastructure"])
        cli = args.cli.resolve()
        env = {**os.environ, "DEVLINE_BIN": str(cli), "DEVLINE_HOME": str(home),
               "DEVLINE_CATALOG": str(ROOT / "data")}
        for stack in stacks:
            for kind in kinds:
                print(f"Checking {stack}/{kind}", flush=True)
                try:
                    verify(cli, catalog, root / f"{stack}-{kind}", env, stack, kind)
                except (AssertionError, KeyError, RuntimeError, OSError, ValueError, subprocess.TimeoutExpired) as error:
                    failures.append(f"{stack}/{kind}")
                    print(f"FAIL {stack}/{kind}: {error}", flush=True)
    if failures:
        raise SystemExit("Failed demos: " + ", ".join(failures))


if __name__ == "__main__":
    main()
