#!/usr/bin/env python3
"""Verify generated Java/.NET reconciliation, issue and approval apps on macOS.

Requires a built Devline CLI, local Maven/JDK and .NET. No Docker or service
images are required. Every project uses a temporary Devline library and its
own embedded database. Network probes require the macOS sandbox boundary.
"""
import argparse
from http.cookiejar import CookieJar, DefaultCookiePolicy
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
from urllib.parse import urlencode, urljoin, urlsplit
from urllib.request import HTTPCookieProcessor, HTTPRedirectHandler, ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parents[1]
STARTERS = {
    "java": {"cli": "json-tool", "api": "spring-api", "mvc": "spring-mvc"},
    "dotnet": {"cli": "json-cli", "api": "aspnet-api", "mvc": "aspnet-mvc"},
}

def run(arguments, *, cwd=None, env=None, input=None, check=True):
    result = subprocess.run(arguments, cwd=cwd, env=env, input=input, text=True,
                            capture_output=True, timeout=600)
    if check and result.returncode:
        raise RuntimeError(f"{' '.join(map(str, arguments))} failed:\n{result.stdout}{result.stderr}")
    return result.stdout if check else result

def need(condition, message):
    if not condition:
        raise AssertionError(message)

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


def verify_assets(browser, page, url):
    for asset in Forms(page).assets:
        target = urljoin(url, asset)
        ensure_local(target, {origin(url)})
        status, headers, source, _ = browser.request(target)
        need(status == 200, "A rendered asset was unavailable offline")
        if headers.get_content_type() == "text/css":
            need(not re.search(r"(?i)@import\b|url\(\s*[\"']?(?:https?:)?//", source),
                 "The stylesheet declares an external asset or import")


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


def verify_cli(project, env):
    run(["./devline.sh", "run"], cwd=project, env=env)
    report = json.loads((project / "reports/reconciliation.json").read_text())
    need([entry["status"] for entry in report["invoices"]] == ["paid", "partial", "unpaid", "overpaid"],
         "The reconciliation demo returned incorrect settlement states")
    need(len(report["unmatched"]) == 1, "Unknown payments must appear in the report")
    invalid = project / "fixtures/invalid.csv"
    invalid.write_text("reference,currency,amount\nBAD,EUR,1.001\n")
    target = project / "reports/invalid.json"
    result = run(["./devline.sh", "exec", "./.devline/scripts/run.sh", "--invoices", str(invalid),
                  "--payments", "fixtures/payments.csv", "--output", str(target)], cwd=project, env=env, check=False)
    need(result.returncode != 0 and not target.exists(), "Invalid money must fail before writing a report")


def verify_api(project, env, base):
    browser = Browser(base)
    status, _, body, _ = browser.request(base + "/")
    need(status == 200 and json.loads(body)["issues"] == "/api/issues", "Open App must show the API index")
    collection = base + "/api/issues"
    need(browser.request(collection, method="POST", payload={"title": " ", "priority": "high"})[0] == 400,
         "A blank issue title must be rejected")
    status, headers, body, _ = browser.request(collection, method="POST", payload={
        "title": "Verify offline search", "description": "Start from an empty cache", "priority": "high"})
    need(status == 201 and headers.get("Location"), "Issue creation must return a resource location")
    issue_path = urlsplit(urljoin(collection, headers["Location"])).path
    change = {"title": "Verify offline search", "description": "Investigating", "priority": "high",
              "status": "resolved", "version": 1}
    need(browser.request(base + issue_path, method="PUT", payload=change)[0] == 409,
         "An issue must enter progress before resolution")
    change["status"] = "in_progress"
    need(browser.request(base + issue_path, method="PUT", payload=change)[0] == 200, "Starting work failed")
    need(browser.request(base + issue_path, method="PUT", payload=change)[0] == 409, "Stale edits must conflict")
    status, _, body, _ = browser.request(collection + "?q=offline&status=in_progress")
    need(status == 200 and len(json.loads(body)) == 1, "Search and state filtering failed")
    run(["./devline.sh", "stop"], cwd=project, env=env)
    base = wait_for_app(project, env)
    browser = Browser(base)
    status, _, body, _ = browser.request(base + issue_path)
    issue = json.loads(body)
    need(status == 200 and issue["version"] == 2 and issue["status"] == "in_progress",
         "Issue state did not survive managed restart")
    status, _, body, _ = browser.request(base + issue_path + "/events")
    need(status == 200 and len(json.loads(body)) == 2, "Issue audit history did not persist")


def sign_in(base, stack, username):
    browser = Browser(base)
    path = "/login" if stack == "java" else "/account/login"
    status, _, page, url = browser.request(base + path)
    need(status == 200, "The local sign-in page was unavailable")
    form = Forms(page).matching("password")
    need(form, "Sign-in must render its CSRF-protected password form")
    action, inputs = form
    status, _, page, url = browser.request(urljoin(url, action), method="POST",
                                         form={**inputs, "username": username, "password": "devline-local"})
    need(status == 200 and urlsplit(url).path == "/", "Demo sign-in did not return to the request queue")
    verify_assets(browser, page, url)
    return browser, page


def verify_mvc(project, env, stack, base):
    alice, page = sign_in(base, stack, "alice")
    creation = Forms(page).matching("title", "amount", target="/requests/create")
    need(creation, "The request queue did not render a purchase form")
    action, fields = creation
    payload = {"title": "Workshop equipment", "amount": "240.50", "reason": "Hands-on learning for the team"}
    need(alice.request(base + action, method="POST", form=payload)[0] in (400, 403), "Missing CSRF token was accepted")
    status, _, detail, url = alice.request(base + action, method="POST", form={**fields, **payload})
    need(status == 200 and "240.50" in detail, "The purchase request was not created")
    path = urlsplit(url).path
    need(alice.request(base + path + "/decision", method="POST", form={
        **fields, "version": "1", "decision": "approved", "note": "My own request"})[0] == 403,
        "A requester cannot approve their own request")
    bob, _ = sign_in(base, stack, "bob")
    status, _, detail, _ = bob.request(base + path)
    decision = Forms(detail).matching("version", target=path + "/decision")
    need(status == 200 and decision, "An independent approver must receive the decision form")
    _, fields = decision
    payload = {**fields, "decision": "approved", "note": "Within the team budget"}
    status, _, detail, _ = bob.request(base + path + "/decision", method="POST", form=payload)
    need(status == 200 and "Within the team budget" in detail, "Approval did not record the decision")
    need(bob.request(base + path + "/decision", method="POST", form={**payload, "version": "2"})[0] == 409,
         "A completed decision must be immutable")
    run(["./devline.sh", "stop"], cwd=project, env=env)
    base = wait_for_app(project, env)
    bob, page = sign_in(base, stack, "bob")
    status, _, detail, url = bob.request(base + path)
    need(status == 200 and "Within the team budget" in detail and "approved" in detail and "submitted" in detail,
         "Purchase state and audit trail did not survive managed restart")
    verify_assets(bob, detail, url)
    logout_path = "/logout" if stack == "java" else "/account/logout"
    logout = Forms(page).matching(target=logout_path)
    need(logout, "The queue must include a sign-out form")
    bob.request(base + logout_path, method="POST", form=logout[1])
    need(300 <= bob.request(base + "/", follow=False)[0] < 400, "Sign-out did not clear the session")


def verify(cli, project, env, stack, kind):
    template = f"{stack}/{STARTERS[stack][kind]}"
    generated = json.loads(run([cli, "--offline", "--json", "new", template, project], env=env))["data"]
    need(generated["ready_offline"] and not generated["services"],
         f"{template} must generate offline without external services")
    try:
        verify_network_boundary(project, env)
        run(["./devline.sh", "build"], cwd=project, env=env)
        if kind == "cli":
            verify_cli(project, env)
        else:
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0))
                config_file = project / ".devline/config.json"
                config = json.loads(config_file.read_text())
                config["run"]["url"] = f"http://127.0.0.1:{reservation.getsockname()[1]}"
                config_file.write_text(json.dumps(config))
            base = wait_for_app(project, env)
            if kind == "api":
                verify_api(project, env, base)
            else:
                verify_mvc(project, env, stack, base)
    finally:
        if kind != "cli":
            run(["./devline.sh", "stop"], cwd=project, env=env)
    print(f"PASS {template}@{generated['version']}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=ROOT / "core/target/release/devline")
    parser.add_argument("--stack", choices=STARTERS, action="append")
    parser.add_argument("--kind", choices=("cli", "api", "mvc"), action="append")
    args = parser.parse_args()
    failures = []
    with tempfile.TemporaryDirectory(prefix="devline java dotnet ") as temporary:
        root = Path(temporary).resolve()
        home = root / "library"
        home.mkdir()
        cli = args.cli.resolve()
        env = {**os.environ, "DEVLINE_BIN": str(cli), "DEVLINE_HOME": str(home), "DEVLINE_CATALOG": str(ROOT / "data")}
        for stack in args.stack or STARTERS:
            for kind in args.kind or ("cli", "api", "mvc"):
                print(f"Checking {stack}/{kind}", flush=True)
                try:
                    verify(cli, root / f"{stack}-{kind}", env, stack, kind)
                except (AssertionError, KeyError, RuntimeError, OSError, ValueError, subprocess.TimeoutExpired) as error:
                    failures.append(f"{stack}/{kind}")
                    print(f"FAIL {stack}/{kind}: {error}", flush=True)
    if failures:
        raise SystemExit("Failed apps: " + ", ".join(failures))

if __name__ == "__main__":
    main()
