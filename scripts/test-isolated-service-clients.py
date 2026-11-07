#!/usr/bin/env python3
"""Add and exercise each Python service client in a separate, empty project.

Connections come from a running Devline service stack; each test receives only
its selected client's packages. No infrastructure credentials are written to
results or logs. The actual Devline CLI prepares and runs every project offline.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SERVICES = ['keycloak','postgres','mysql','rabbitmq','redis','mongodb','nats','mailpit','meilisearch','seaweedfs']


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--stack-project',type=Path,required=True)
    p.add_argument('--home',type=Path,required=True)
    p.add_argument('--cli',type=Path,default=ROOT/'core/target/debug/devline')
    p.add_argument('--catalog',type=Path,default=ROOT/'data')
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--service',action='append',choices=SERVICES)
    args=p.parse_args(); output=args.output.resolve();output.mkdir(parents=True,exist_ok=True)
    base=[str(args.cli.resolve()),'--offline','--home',str(args.home.resolve()),'--catalog',str(args.catalog.resolve())]
    # Capture credentials in memory only. They are injected using the runner's
    # standard environment contract, never copied to a public fixture/report.
    configuration=subprocess.check_output(base+['--json','infra','env',str(args.stack_project.resolve())],text=True)
    data=json.loads(configuration);data=data.get('data',data);connections=data['environment']
    results=[]
    for service in args.service or SERVICES:
        directory=output/service
        if directory.exists():raise SystemExit('Choose a fresh output directory: '+str(directory))
        project=directory/'project';project.mkdir(parents=True)
        (project/'requirements.txt').write_text('')
        (project/'probe.py').write_text('# Fresh project used to prove one explicitly selected client.\n')
        home=directory/'empty-home';home.mkdir()
        temporary=directory/'tmp';temporary.mkdir()
        env={'HOME':str(home),'TMPDIR':str(temporary),'PATH':os.environ.get('PATH','/usr/bin:/bin'), 'DEVKIT_PYTHON3':shutil.which('python3'),'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1', **connections}
        def run(arguments,label):
            log=directory/(label+'.log')
            with log.open('wb') as stream:
                result=subprocess.run(base+arguments,cwd=project,env=env,stdin=subprocess.DEVNULL,stdout=stream,stderr=subprocess.STDOUT,timeout=180)
            if result.returncode:raise RuntimeError(label+' failed; inspect '+str(log))
        started=time.monotonic()
        try:
            run(['--json','infra','add',str(project),'--client','python:'+service],'add-selected-client')
            if service=='mailpit':
                # Built-in SMTP intentionally adds no dependencies. Prepare its
                # empty native pip graph before exercising the common runner.
                managed=project/'.devline-offline';(managed/'wheelhouse').mkdir(parents=True)
                (managed/'requirements.lock').write_text('')
                run(['--json','project','--path',str(project),'prepare'],'verify-empty-stdlib-project')
            proof=directory/'live.json'
            run(['project','--path',str(project),'run','--','python3',str(ROOT/'scripts/test-live-service-clients.py'),'--probe','--service',service,'--assert-isolated','--output',str(proof)],'live-selected-client')
            evidence=json.loads(proof.read_text())
            assert evidence['unselected_primary_clients_absent'] is True
            assert len(evidence['results'])==1 and evidence['results'][0]['status']=='passed'
            result={'service':service,'status':'passed','selection':['python:'+service],'packages':evidence['packages'],'network_boundary':evidence['network_boundary'],'unselected_primary_clients_absent':True,'operations':evidence['results'][0]['operations'],'cleanup':evidence['results'][0]['cleanup']}
            print('PASS isolated '+service,flush=True)
        except Exception as error:
            result={'service':service,'status':'failed','error':str(error)}
            print('FAIL isolated '+service+': '+str(error),flush=True)
        result['seconds']=round(time.monotonic()-started,2);results.append(result)
        report={'schema_version':1,'measured_at':datetime.now(timezone.utc).isoformat(),'scope':'Independent Python client selections: separate empty projects, actual offline CLI dependency additions, fresh runner environments, and live service operations. Every unselected primary client is confirmed absent.','catalog_sha256':hashlib.sha256((args.catalog/'service-clients/python.json').read_bytes()).hexdigest(),'results':results}
        (output/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    return 0 if all(row['status']=='passed' for row in results) else 1


if __name__=='__main__':raise SystemExit(main())
