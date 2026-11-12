#!/usr/bin/env python3
"""Create and exercise the curated application catalog through Devline on macOS.

Each generated project gets its actual relocated scripts and dependency payload.
Creation runs tests behind Devline's outbound network boundary. Web apps are started on a
free loopback port, fetched, stopped, and restarted. CLI demos run to completion.
Desktop apps are built by their offline tests; interactive UI review is separate.
"""

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time
from urllib.request import ProxyHandler, build_opener

ROOT = Path(__file__).resolve().parents[1]
RUNTIMES = {"dotnet", "elixir", "go", "java", "node", "python", "ruby", "rust", "swift"}


def command(args, *, cwd, env, log, timeout=900):
    with log.open("w") as output:
        process = subprocess.Popen(
            [str(a) for a in args],
            cwd=cwd,
            env=env,
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            result = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            raise RuntimeError(f"Timed out after {timeout}s; see {log}") from None
    if result:
        tail = log.read_text(errors="replace")[-5000:]
        raise RuntimeError(f"Exit {result}: {args}\n{tail}\nLog: {log}")
    return log.read_text(errors="replace")


def available_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def validate_payload(entry):
    project = ROOT / "data" / entry["path"] / "project"
    generated = [
        ".build",
        ".venv",
        "_build",
        "target",
        "obj",
        "dist",
        ".data",
        "test-results",
        "__pycache__",
        ".pytest_cache",
        "node_modules/.vite",
    ]
    if entry["stack"] == "dotnet":
        generated.append("bin")
    if entry["id"] == "rust/tauri-desktop":
        generated.append("gen")
    leftovers = [str(project / name) for name in generated if (project / name).exists()]
    if leftovers:
        raise AssertionError(
            "Remove generated source-payload artifacts: " + ", ".join(leftovers)
        )


def verify(cli, entry, output, env):
    validate_payload(entry)
    slug = entry["id"].replace("/", "-")
    project = output / "projects" / slug
    logs = output / "logs" / slug
    logs.mkdir(parents=True)
    exact = f"{entry['id']}@{entry['version']}"
    print(f"CREATE {exact}", flush=True)
    raw = command(
        [cli, "--offline", "--json", "new", exact, project],
        cwd=ROOT,
        env=env,
        log=logs / "create.log",
        timeout=1800,
    )
    created = json.loads(raw)["data"]
    if created.get("template") != entry["id"] or not created.get("ready_offline"):
        raise AssertionError(f"Creation did not qualify {exact}: {created}")
    config_file = project / ".devline/config.json"
    config = json.loads(config_file.read_text())
    if not (project / "README.md").is_file() or not config.get("run", {}).get(
        "command"
    ):
        raise AssertionError("Every app needs a guide and a runnable workflow")
    # Check the actual kernel denial, not merely failure to reach an external URL.
    probe = (
        "import errno,socket; s=socket.socket(); s.settimeout(3); "
        'result=s.connect_ex(("1.1.1.1",443)); s.close(); '
        'print("network errno",result); '
        'assert result in (errno.EPERM,errno.EACCES), "External networking was not denied"'
    )
    command(
        [
            cli,
            "--offline",
            "project",
            "--path",
            project,
            "run",
            "--",
            "python3",
            "-c",
            probe,
        ],
        cwd=project,
        env=env,
        log=logs / "network-boundary.log",
        timeout=600,
    )
    # `new` invokes verify_generated: the relocated project is tested with isolated
    # caches and external networking denied before it can be published. Check that
    # receipt against the actual generated files instead of running the suite twice.
    readiness = json.loads(
        command(
            [cli, "--offline", "--json", "project", "--path", project, "readiness"],
            cwd=project,
            env=env,
            log=logs / "readiness.log",
            timeout=120,
        )
    )["data"]
    if not readiness.get("ready_offline"):
        raise AssertionError(
            f"Generated verification receipt is not current: {readiness}"
        )
    url = config["run"].get("url")
    if url:
        base = f"http://127.0.0.1:{available_port()}"
        config["run"]["url"] = base
        config_file.write_text(json.dumps(config, indent=2) + "\n")
        path = config["run"].get("health_path", "/")
        opener = build_opener(ProxyHandler({}))
        for attempt in range(2):
            try:
                command(
                    [cli, "--offline", "--json", "project", "--path", project, "start"],
                    cwd=project,
                    env=env,
                    log=logs / f"start-{attempt}.log",
                    timeout=180,
                )
                deadline = time.monotonic() + 600
                last_error = None
                while time.monotonic() < deadline:
                    try:
                        with opener.open(base + path, timeout=3) as response:
                            if response.status != 200:
                                raise OSError(f"Unhealthy response {response.status}")
                            response.read(1_000_000)
                        break
                    except OSError as error:
                        last_error = error
                        time.sleep(0.5)
                else:
                    command(
                        [cli, "--offline", "project", "--path", project, "logs"],
                        cwd=project,
                        env=env,
                        log=logs / f"app-{attempt}.log",
                        timeout=30,
                    )
                    raise AssertionError(
                        f"App did not become healthy: {last_error}; see {logs}"
                    )
            finally:
                command(
                    [cli, "--offline", "project", "--path", project, "stop"],
                    cwd=project,
                    env=env,
                    log=logs / f"stop-{attempt}.log",
                    timeout=45,
                )
    elif entry["app_type"] != "desktop":
        command(
            [cli, "--offline", "project", "--path", project, "run"],
            cwd=project,
            env=env,
            log=logs / "demo.log",
            timeout=180,
        )
    print(f"PASS   {exact}", flush=True)
    return {
        "id": entry["id"],
        "version": entry["version"],
        "status": "passed",
        "project": str(project),
        "logs": str(logs),
        "checks": ["generated project", "kernel network denial", "offline tests"]
        + (
            ["managed start, health, stop, restart"]
            if url
            else ["desktop build (interactive review separate)"]
            if entry["app_type"] == "desktop"
            else ["default demo"]
        ),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cli", type=Path, default=ROOT / "core/target/release/devline"
    )
    parser.add_argument("--stack", choices=sorted(RUNTIMES), action="append")
    parser.add_argument("--id", action="append", dest="ids")
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT
        / ".build"
        / "representative-apps"
        / datetime.now().strftime("generated-%Y%m%d-%H%M%S"),
    )
    args = parser.parse_args()
    catalog = json.loads((ROOT / "data/templates-catalog.json").read_text())
    entries = [
        entry
        for entry in catalog
        if (not args.stack or entry["stack"] in args.stack)
        and (not args.ids or entry["id"] in args.ids)
    ]
    if not entries:
        parser.error("No catalog entries match the selection")
    if len({entry["id"] for entry in entries}) != len(entries):
        parser.error(
            "The representative catalog must contain one current release per app"
        )
    if not args.ids:
        for stack in args.stack or sorted(RUNTIMES):
            count = sum(entry["stack"] == stack for entry in entries)
            if not 3 <= count <= 5:
                parser.error(f"{stack} has {count} apps; expected 3–5")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    (output / "projects").mkdir()
    cli = args.cli.resolve()
    env = {
        **os.environ,
        "DEVLINE_BIN": str(cli),
        "DEVLINE_HOME": str(output / "library"),
        "DEVLINE_CATALOG": str(ROOT / "data"),
        "CARGO_BUILD_JOBS": "2",
        "SWIFT_BUILD_JOBS": "2",
        "ERL_FLAGS": "+S 2:2",
    }
    results = []
    started = time.monotonic()
    for entry in entries:
        try:
            results.append(verify(cli, entry, output, env))
        except (AssertionError, RuntimeError, OSError, ValueError, KeyError) as error:
            results.append({"id": entry["id"], "status": "failed", "error": str(error)})
            print(f"FAIL   {entry['id']}: {error}", flush=True)
        (output / "results.json").write_text(
            json.dumps(
                {
                    "checked_at": datetime.now(timezone.utc).isoformat(),
                    "elapsed_seconds": round(time.monotonic() - started, 1),
                    "results": results,
                },
                indent=2,
            )
            + "\n"
        )
    print(f"Results: {output / 'results.json'}", flush=True)
    if any(result["status"] != "passed" for result in results):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
