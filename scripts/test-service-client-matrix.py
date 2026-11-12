#!/usr/bin/env python3
"""Audit each service client independently on every latest bundled starter.

Without --run, list the matrix without creating projects. A run retains its logs,
JSON report, baselines, and failed projects in a fresh output directory. Successful
project clones are removed unless --keep-successes is supplied.

Examples:
    python3 scripts/test-service-client-matrix.py
    python3 scripts/test-service-client-matrix.py --stack java
    python3 scripts/test-service-client-matrix.py --run --snapshot-catalog \
        --stack python --expected-per-stack 3 --output /tmp/devline-python-matrix

Every runtime gets an isolated Devline library with no package shelf. The CLI's
--offline add path denies external networking while resolving and verifying the
project. --wrapper-test additionally runs the starter's test launcher with the
same network boundary and explicit isolated Devline environment.
"""
import argparse
import datetime
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import signal
import stat
import subprocess
import time
import tempfile

ROOT = Path(__file__).resolve().parents[1]
STACKS = ['node', 'python', 'go', 'rust', 'dotnet', 'ruby', 'elixir', 'swift', 'java']

def latest(catalog):
    releases = {}
    for item in json.loads((catalog / 'templates-catalog.json').read_text()):
        version = tuple(int(n) for n in item['version'].split('.'))
        if item['id'] not in releases or version > releases[item['id']][0]:
            releases[item['id']] = (version, item)
    return [v[1] for v in releases.values()]

def remove_tree(path):
    """Delete an isolated audit tree, including Go's read-only module directories."""
    path = Path(path)
    if path.is_symlink():
        path.unlink()
        return
    # Make directories traversable/deletable without following source symlinks.
    path.chmod(path.stat().st_mode | stat.S_IRWXU)
    for current, directories, _ in os.walk(path, followlinks=False):
        for name in directories:
            directory = Path(current) / name
            if not directory.is_symlink():
                directory.chmod(directory.stat().st_mode | stat.S_IRWXU)
    shutil.rmtree(path)

def clone(source, destination):
    if destination.exists():
        raise RuntimeError(f'Refusing to overwrite {destination}')
    result = subprocess.run(['/bin/cp', '-cR', str(source), str(destination)], capture_output=True)
    if result.returncode:
        if destination.exists(): remove_tree(destination)
        shutil.copytree(source, destination, symlinks=True)

def response(text):
    """Read the final CLI envelope even if logs precede it or it is pretty-printed."""
    decoder = json.JSONDecoder()
    found = None
    for match in re.finditer(r'(?m)^\s*(?=\{)', text):
        try:
            value, _ = decoder.raw_decode(text, match.end())
        except ValueError:
            continue
        if isinstance(value, dict) and value.get('schema_version') == 1 and ('data' in value or 'error' in value):
            found = value
    return found

def run(command, log, timeout, cwd=None, env=None):
    started = time.monotonic()
    with log.open('wb') as stream:
        p = subprocess.Popen(command, cwd=cwd, env=env, stdout=stream, stderr=subprocess.STDOUT,
                             stdin=subprocess.DEVNULL, start_new_session=True)
        try:
            p.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGTERM)
            try: p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(p.pid, signal.SIGKILL); p.wait()
            return {'exit_code': None, 'timed_out': True, 'seconds': round(time.monotonic()-started, 2)}
    result = {'exit_code': p.returncode, 'seconds': round(time.monotonic()-started, 2)}
    envelope = response(log.read_text(errors='replace'))
    if envelope is not None:
        result.update({key: envelope[key] for key in ('data', 'error') if key in envelope})
    return result

def main():
    p=argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('--stack', action='append', choices=STACKS)
    p.add_argument('--starter', action='append', help='Exact starter id; latest bundled version is used')
    p.add_argument('--expected-per-stack', type=int, help='Refuse a run until every selected stack has this many latest starters')
    p.add_argument('--client', action='append', help='Service id, e.g. postgres')
    p.add_argument('--catalog', type=Path, default=ROOT/'data')
    p.add_argument('--cli', type=Path, default=ROOT/'core/target/release/devline')
    p.add_argument('--output', type=Path, help='Fresh or empty output directory; defaults to a new isolated temporary directory')
    p.add_argument('--seed-infrastructure', type=Path, help='Isolated library infrastructure directory to clone; never a shelf')
    p.add_argument('--timeout', type=int, default=1200)
    p.add_argument('--keep-successes', action='store_true')
    p.add_argument('--wrapper-test', action='store_true', help='Also run ./devline.sh test with outbound networking denied')
    p.add_argument('--snapshot-catalog', action='store_true', help='Clone the entire catalog and CLI into this run before auditing, so concurrent repository edits cannot affect it')
    p.add_argument('--run', action='store_true')
    args=p.parse_args()
    catalog=args.catalog.resolve(); cli=args.cli.resolve()
    output=(args.output.resolve() if args.output else
            Path(tempfile.mkdtemp(prefix='devline-client-matrix-')).resolve() if args.run else None)
    source_catalog=catalog; source_cli=cli
    if output is not None and (output == Path.home() or output.is_relative_to(Path.home()/'Library/Application Support/DevKit')):
        raise SystemExit('Choose an isolated output directory.')
    if args.run and output.exists() and any(output.iterdir()):
        raise RuntimeError('Choose a fresh or empty output directory; existing audit data will not be overwritten')
    if args.run:
        print('Audit output: '+str(output), flush=True)
    if args.run and args.snapshot_catalog:
        if output.is_relative_to(catalog): raise RuntimeError('Snapshot output must be outside the source catalog')
        # Capture a matching source/generated catalog pair before cloning. The selected
        # starter owners must have finished writing their source trees before this run.
        pair={name:(catalog/name).read_bytes() for name in ['templates-catalog.json','catalog.json']}
        if json.loads(pair['templates-catalog.json'])!=json.loads(pair['catalog.json'])['templates']:
            raise RuntimeError('Source/generated catalogs are not synchronized; wait for a stable catalog pair')
        output.mkdir(parents=True, exist_ok=True)
        frozen=output/'catalog'
        print('Cloning catalog into '+str(frozen), flush=True)
        clone(catalog,frozen)
        for name,contents in pair.items():(frozen/name).write_bytes(contents)
        catalog=frozen
        (output/'bin').mkdir(exist_ok=True)
        cli=output/'bin/devline'
        shutil.copy2(source_cli,cli)
        print('Frozen catalog and CLI are ready.', flush=True)
    starters=[r for r in latest(catalog) if r['stack'] in (args.stack or STACKS)
              and (not args.starter or r['id'] in args.starter)]
    if not starters: raise RuntimeError('No starters match this selection')
    clients={s: [r for r in json.loads((catalog/'service-clients'/f'{s}.json').read_text())['integrations']
                 if not args.client or r['service'] in args.client] for s in {r['stack'] for r in starters}}
    matrix=[(r,c) for r in starters for c in clients[r['stack']]]
    print(f'{len(starters)} latest starters, {len(matrix)} independent client additions', flush=True)
    if not args.run:
        for r,c in matrix: print(f"{r['id']}@{r['version']} {c['id']}")
        return 0
    if args.expected_per_stack is not None:
        counts={stack:sum(r['stack']==stack for r in starters) for stack in (args.stack or STACKS)}
        if any(count!=args.expected_per_stack for count in counts.values()):
            raise RuntimeError(f'The starter rewrite is not ready for this audit: expected {args.expected_per_stack} per stack, found {counts}')
    snapshot={name:(catalog/name).read_bytes() for name in ['templates-catalog.json','catalog.json']}
    if json.loads(snapshot['templates-catalog.json'])!=json.loads(snapshot['catalog.json'])['templates']:
        raise RuntimeError('The generated catalog does not match templates-catalog.json; synchronize it first')
    def unchanged_catalog():
        if any((catalog/name).read_bytes()!=contents for name,contents in snapshot.items()):
            raise RuntimeError('Catalog changed during the audit; stop and use a fresh output after the rewrite completes')
    output.mkdir(parents=True, exist_ok=True)
    if (output/'results.json').exists(): raise RuntimeError('Choose a fresh output; an audit report already exists here')
    results=[]
    report={'schema_version':1,'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'catalog':str(catalog),'cli':str(cli),'source_catalog':str(source_catalog),'source_cli':str(source_cli),'cli_sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'scope':'latest bundled starter releases; each client alone on a fresh clone; library has no package shelf',
            'network_boundary':'CLI --offline stages additions in a fresh HOME with sandbox-exec external network-outbound denial, then verifies before publication',
            'catalog_sha256': hashlib.sha256(snapshot['templates-catalog.json']).hexdigest(),
            'generated_catalog_sha256': hashlib.sha256(snapshot['catalog.json']).hexdigest(),
            'expected_cases':len(matrix),'results':results}
    def save():
        temp=output/'results.tmp.json'; temp.write_text(json.dumps(report, indent=2)+'\n');temp.replace(output/'results.json')
    for starter in starters:
        unchanged_catalog()
        stack=starter['stack']; section=output/stack; section.mkdir(exist_ok=True)
        library=section/'library'; library.mkdir(exist_ok=True)
        if (library/'packages').exists(): raise RuntimeError('Audit library must not contain any package shelf')
        if args.seed_infrastructure and not (library/'infrastructure').exists():
            clone(args.seed_infrastructure.resolve(),library/'infrastructure')
        command=[str(cli),'--json','--offline','--home',str(library),'--catalog',str(catalog)]
        stem=starter['id'].split('/')[-1]
        folder=section/stem; folder.mkdir(exist_ok=True)
        baseline=folder/'baseline'
        baseline_log=folder/'create.jsonl'
        if not baseline.exists():
            creation=run(command+['new',f"{starter['id']}@{starter['version']}",str(baseline)],baseline_log,args.timeout)
        else:
            creation={'exit_code':0,'reused':True}
        if creation.get('exit_code') != 0:
            for client in clients[stack]:
                results.append({'starter':starter['id'],'version':starter['version'],'client':client['id'],
                                'status':'baseline_failed','creation':creation,'log':str(baseline_log)})
            print('BASELINE FAILED '+starter['id'],flush=True);save();continue
        for client in clients[stack]:
            unchanged_catalog()
            case=folder/client['service']; case.mkdir(exist_ok=True)
            project=case/'project'
            if project.exists(): raise RuntimeError(f'Choose a fresh output or remove a previous case first: {project}')
            clone(baseline,project)
            result=run(command+['infra','add',str(project),'--client',client['id']],case/'add.jsonl',args.timeout)
            data=result.get('data',{}); state=data.get('status')
            success=(result.get('exit_code')==0 and data.get('complete') is True
                     and data.get('clients')==[client['id']]
                     and (state=='already_included' or state=='ready_offline' and data.get('ready_offline') is True))
            row={'starter':starter['id'],'version':starter['version'],'bundled_revision':starter.get('bundled_revision',1),'name':starter['name'],'client':client['id'],
                 'status':'passed' if success else 'failed','addition':result,
                 'mutation': 'already_included' if state=='already_included' else 'changed' if data.get('changed') else 'none',
                 'log':str(case/'add.jsonl')}
            if success:
                declarations=run(command+['infra','clients',str(project)],case/'clients.jsonl',60)
                services=declarations.get('data',{}).get('services',[])
                chosen=[lib for service in services for lib in service.get('libraries',[]) if lib.get('id')==client['id']]
                row['selected_client_installed']=bool(declarations.get('exit_code')==0 and chosen and chosen[0].get('installed'))
                if not row['selected_client_installed']:
                    row['status']='declaration_failed'
                if args.wrapper_test:
                    policy='(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))'
                    wrapper_env=dict(os.environ, DEVLINE_BIN=str(cli), DEVLINE_HOME=str(library), DEVLINE_CATALOG=str(catalog), DEVLINE_OFFLINE='true')
                    wrapper=run(['/usr/bin/sandbox-exec','-p',policy,'/bin/sh','./devline.sh','test'],case/'wrapper-test.log',args.timeout,cwd=project,env=wrapper_env)
                    row['wrapper_test']=wrapper
                    if wrapper.get('exit_code')!=0:row['status']='wrapper_failed'
            if (library/'packages').exists(): raise RuntimeError('Package shelf appeared during audit')
            results.append(row);save()
            print(f"{row['status'].upper()} {starter['id']} {client['id']} {result['seconds']}s",flush=True)
            if row['status']=='passed' and not args.keep_successes:remove_tree(project)
    report['finished_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();save()
    return 0 if len(results)==len(matrix) and all(r['status']=='passed' for r in results) else 1

if __name__=='__main__':raise SystemExit(main())
