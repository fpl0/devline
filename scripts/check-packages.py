#!/usr/bin/env python3
"""Check the package shelf catalogs in data/packages against their schema."""
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent / 'data' / 'packages'
STACKS = ['dotnet', 'elixir', 'go', 'java', 'node', 'python', 'ruby', 'rust', 'swift']
CATEGORIES = {'Web', 'HTTP', 'Database', 'Data', 'Serialization', 'Testing', 'CLI', 'Async',
              'Logging', 'Security', 'Cloud', 'AI', 'UI', 'Tooling', 'Utilities'}
# The same characters the core accepts before a name reaches a package manager.
NAME = re.compile(r'[A-Za-z0-9@/._:\[\],+-]{1,200}')
EXPECTED = 100


def check(stack):
    problems = []
    path = ROOT / f'{stack}.json'
    if not path.is_file():
        return [f'{stack}: missing {path.name}']
    data = json.loads(path.read_text())
    if data.get('schema_version') != 1 or data.get('stack') != stack or not data.get('registry'):
        problems.append(f'{stack}: schema_version, stack or registry is wrong')
    packages = data.get('packages', [])
    if len(packages) != EXPECTED:
        problems.append(f'{stack}: {len(packages)} packages, expected {EXPECTED}')
    seen = set()
    for package in packages:
        name = package.get('name', '')
        where = f'{stack}: {name or "(unnamed)"}'
        if not NAME.fullmatch(name) or name.startswith('-'):
            problems.append(f'{where}: name is not safe to pass to a package manager')
        if name.lower() in seen:
            problems.append(f'{where}: listed twice')
        seen.add(name.lower())
        summary = package.get('summary', '')
        if not summary or len(summary) > 90:
            problems.append(f'{where}: summary must be 1 to 90 characters')
        if package.get('category') not in CATEGORIES:
            problems.append(f'{where}: unknown category {package.get("category")!r}')
        if stack == 'java' and name.count(':') != 1:
            problems.append(f'{where}: expected groupId:artifactId')
        if stack == 'swift':
            url = package.get('url', '')
            if not (url.startswith('https://') and url.endswith('.git')):
                problems.append(f'{where}: url must be an https git URL')
            if url.rsplit('/', 1)[-1].removesuffix('.git') != name:
                problems.append(f'{where}: name must be the repository name')
            if not package.get('products') or not all(isinstance(p, str) and p for p in package['products']):
                problems.append(f'{where}: products must list library product names')
        if 'imports' in package and (stack != 'go' or not package['imports']):
            problems.append(f'{where}: imports is a non-empty list for go only')
        if 'features' in package and stack != 'rust':
            problems.append(f'{where}: features apply to rust only')
    return problems


def main():
    problems = [problem for stack in STACKS for problem in check(stack)]
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        raise SystemExit(1)
    print(f'Verified {len(STACKS)} package shelf catalogs, {EXPECTED} packages each')


if __name__ == '__main__':
    main()
