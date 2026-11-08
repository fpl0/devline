#!/bin/sh
set -eu
mode=${1:?prepare, verify, or run}
project=${2:?staged project}
cache=${3:?staged cache}
tool=${4:?Python executable}
shift 4
cd "$project"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-python.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
"$tool" -I -m venv "$scratch/venv"
python="$scratch/venv/bin/python"
wheelhouse="$cache/wheelhouse"
lockfile="$cache/requirements.lock"
if [ "$mode" = run ] && [ ! -f "$lockfile" ] && { [ -f "$project/.devline/origin.json" ] || [ -f "$project/.devkit-origin.json" ]; }; then
  wheelhouse="$project/.devline/packages/wheelhouse"
  [ -d "$wheelhouse" ] || wheelhouse="$project/wheelhouse"
  lockfile="$project/requirements.txt"
fi

pip() {
  "$python" -I -m pip --isolated --disable-pip-version-check --no-cache-dir "$@"
}

install_wheels() {
  pip install --no-index --find-links="$wheelhouse" --only-binary=:all: --require-hashes -r "$lockfile"
  pip check
}

case "$mode" in
  prepare)
    mkdir -p "$cache/wheelhouse"
    if [ -f uv.lock ]; then
      command -v uv >/dev/null 2>&1 || { echo 'Install uv before preparing a project with uv.lock.' >&2; exit 1; }
      uv --no-cache export --locked --no-emit-project --no-editable --no-header --no-annotate \
        --format requirements-txt --output-file "$cache/source-requirements.txt"
      source="$cache/source-requirements.txt"
    elif [ -f requirements.lock ]; then
      source="$project/requirements.lock"
    elif [ -f requirements.txt ]; then
      source="$project/requirements.txt"
    elif [ -f requirements.in ]; then
      source="$project/requirements.in"
    elif [ -f pyproject.toml ] && [ ! -f poetry.lock ] && [ ! -f Pipfile.lock ]; then
      source="$cache/source-requirements.txt"
      "$python" -I - "$source" <<'PYTHON'
import sys
import tomllib
from pathlib import Path

config = tomllib.loads(Path("pyproject.toml").read_text())
project = config.get("project")
if project is None or "dependencies" in project.get("dynamic", []):
    raise SystemExit("Export a static requirements.txt for this project's computed or tool-specific dependencies.")
deps = project.get("dependencies", [])
for group in ("test", "dev"):
    deps.extend(project.get("optional-dependencies", {}).get(group, []))
    for requirement in config.get("dependency-groups", {}).get(group, []):
        if not isinstance(requirement, str):
            raise SystemExit("Export requirements.txt for dependency groups with includes.")
        deps.append(requirement)
Path(sys.argv[1]).write_text("\n".join(dict.fromkeys(deps)) + "\n")
PYTHON
    else
      echo 'Export exact requirements.txt from this project’s package manager before preparing its wheelhouse.' >&2
      exit 1
    fi

    "$python" -I - "$project" "$source" <<'PYTHON'
import re
import shlex
import sys
from pathlib import Path
from pip._vendor.packaging.requirements import InvalidRequirement, Requirement

root = Path(sys.argv[1]).resolve()
seen = set()

def validate(file):
    file = file.resolve(strict=True)
    if not file.is_relative_to(root) or not file.is_file():
        raise SystemExit("Requirements includes must stay inside the project.")
    if file in seen:
        return
    seen.add(file)
    for raw in file.read_text(encoding="utf-8-sig").replace("\\\n", " ").splitlines():
        line = re.split(r"\s+#", raw.strip(), maxsplit=1)[0].strip()
        if not line or line.startswith("#"):
            continue
        include = re.fullmatch(r"(?:-r|-c|--requirement|--constraint)(?:\s+|=)(.+)", line)
        if include:
            targets = shlex.split(include[1])
            if len(targets) != 1 or targets[0].startswith(("-", "/")):
                raise SystemExit("Requirements includes must use a single relative file path.")
            validate(file.parent / targets[0])
            continue
        requirement = re.split(r"\s+--hash(?:=|\s)", line, maxsplit=1)[0]
        try:
            parsed = Requirement(requirement)
        except InvalidRequirement:
            raise SystemExit("Use registry package requirements. Export editable, custom-index, or local dependencies as pinned wheels first.") from None
        if parsed.url:
            raise SystemExit("Direct URL and VCS dependencies need a locally prepared wheel and a registry-style requirement.")

validate(Path(sys.argv[2]))
PYTHON

    if ! pip download --only-binary=:all: --dest "$cache/wheelhouse" -r "$source"; then
      echo 'A dependency could not be resolved as a wheel for this Python and Mac. Check the lock, private registry access, and availability of Apple Silicon wheels.' >&2
      exit 1
    fi
    "$python" -I - "$cache" <<'PYTHON'
from email.parser import BytesParser
from hashlib import file_digest
from pathlib import Path
from sys import argv
from zipfile import ZipFile

cache = Path(argv[1])
with (cache / "requirements.lock").open("w", encoding="utf-8") as output:
    for wheel in sorted((cache / "wheelhouse").glob("*.whl")):
        with ZipFile(wheel) as archive:
            name = next(name for name in archive.namelist() if name.endswith(".dist-info/METADATA"))
            metadata = BytesParser().parsebytes(archive.read(name), headersonly=True)
        with wheel.open("rb") as stream:
            digest = file_digest(stream, "sha256").hexdigest()
        print(f"{metadata['Name']}=={metadata['Version']} --hash=sha256:{digest}", file=output)
PYTHON
    ;;
  verify|run)
    if [ ! -f "$lockfile" ] || [ ! -d "$wheelhouse" ]; then
      echo 'Prepare this project’s Python dependencies before running offline.' >&2
      exit 1
    fi
    export PATH="$scratch/venv/bin:$PATH" PYTHONDONTWRITEBYTECODE=1
    if [ "$mode" = run ]; then
      [ "$#" -gt 0 ] || { echo 'Choose a command to run.' >&2; exit 2; }
      install_wheels >&2
      "$@"
      exit
    fi
    install_wheels
    if "$python" -I -c 'from importlib.util import find_spec; raise SystemExit(find_spec("pytest") is None)' &&
      { [ -d tests ] || "$python" -I -c 'from pathlib import Path; raise SystemExit(not any(Path(".").glob("test_*.py")))'; }; then
      "$python" -B -m pytest -q -p no:cacheprovider
    else
      if [ -d tests ]; then "$python" -B -m unittest discover -s tests; fi
      "$python" -I - <<'PYTHON'
from pathlib import Path

paths = sorted({*Path(".").glob("*.py"), *Path("src").rglob("*.py"), *Path("app").rglob("*.py")})
for path in paths:
    compile(path.read_bytes(), str(path), "exec")
print(f"Offline wheel installation verified. Checked syntax for {len(paths)} Python files.")
PYTHON
    fi
    ;;
  *) echo 'Expected prepare, verify, or run.' >&2; exit 2 ;;
esac
