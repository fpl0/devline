#!/usr/bin/env python3
"""Exercise selected client additions and native offline verification in clean projects."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('stack', choices=['ruby', 'java', 'elixir', 'swift'])
parser.add_argument('--keep', action='store_true')
args = parser.parse_args()
stack = args.stack
catalog = root / 'data'
records = json.loads((catalog / 'service-clients' / (stack + '.json')).read_text())
tool = shutil.which({'ruby':'ruby','java':'mvn','elixir':'elixir','swift':'swift'}[stack])
if not tool: raise SystemExit('Required local runtime is unavailable: ' + stack)
folder = Path(tempfile.mkdtemp(prefix='devline-client-proof-' + stack + '-'))
try:
    project = folder / 'project'
    project.mkdir()
    temporary = folder / 'tmp'; temporary.mkdir()
    home = folder / 'empty-home'; home.mkdir()
    if stack == 'ruby':
        (project / 'Gemfile').write_text('source "https://rubygems.org"\n')
        (project / 'test_clients.rb').write_text('require "bundler/setup"\n'+''.join('require '+json.dumps(module)+'\n' for module in ['pg','trilogy','bunny','redis','mongo','nats/client','net/smtp','meilisearch','aws-sdk-s3','openid_connect'])+'puts "All Ruby service clients load offline."\n')
    elif stack == 'java':
        source = catalog / 'service-clients/artifacts/java'
        (project / 'pom.xml').write_text('<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion><groupId>dev.devline</groupId><artifactId>offline-proof</artifactId><version>1.0.0</version><properties><maven.compiler.release>17</maven.compiler.release><project.build.sourceEncoding>UTF-8</project.build.sourceEncoding></properties><build><plugins><plugin><groupId>org.apache.maven.plugins</groupId><artifactId>maven-compiler-plugin</artifactId><version>3.14.1</version></plugin></plugins></build></project>')
        shutil.copytree(source / 'src', project / 'src')
    elif stack == 'elixir':
        (project / 'mix.exs').write_text('defmodule ClientProof.MixProject do\n use Mix.Project\n def project, do: [app: :client_proof, version: "1.0.0", deps: []]\n def application, do: [extra_applications: [:logger]]\nend\n')
        (project / 'test').mkdir()
        (project / 'test/test_helper.exs').write_text('ExUnit.start()\n')
        (project / 'test/clients_test.exs').write_text('defmodule ClientsTest do\n use ExUnit.Case\n test "all prepared service clients load" do\n  for module <- [Postgrex, MyXQL, AMQP.Connection, Redix, Mongo, Gnat, :gen_smtp_client, Req, ExAws.S3, ExAws.Request.Hackney, SweetXml, Jason, :oidcc] do\n   assert Code.ensure_loaded?(module), inspect(module)\n  end\n end\nend\n')
    else:
        (project / 'Package.swift').write_text('// swift-tools-version: 6.1\nimport PackageDescription\nlet package = Package(name: "ClientProof", platforms: [.macOS(.v14)], targets: [.executableTarget(name: "ClientProof")])\n')
        (project / 'Sources/ClientProof').mkdir(parents=True)
        shutil.copyfile(catalog / 'service-clients/artifacts/swift/Sources/DevlineClients/main.swift', project / 'Sources/ClientProof/main.swift')
    selection = folder / 'selection.json'
    records.update(catalog_dir=str(catalog), offline=True)
    selection.write_text(json.dumps(records))
    environment = os.environ.copy()
    environment.update(HOME=str(home), TMPDIR=str(temporary)+'/', DEVLINE_OFFLINE='1', GIT_TERMINAL_PROMPT='0')
    policy = '(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))'
    prefix = ['/usr/bin/sandbox-exec','-p',policy]
    def run(command, timeout=600):
        subprocess.run(prefix + command, cwd=project, env=environment, check=True, timeout=timeout)
    run(['/bin/sh',str(catalog / 'service-clients' / (stack+'.sh')),str(project),str(selection),'true',tool])
    run(['/bin/sh',str(catalog / 'project-adapters' / (stack+'.sh')),'verify',str(project),str(project / '.devline-offline'),tool])
    if stack == 'ruby':
        run(['/bin/sh',str(catalog / 'project-adapters/ruby.sh'),'run',str(project),str(project / '.devline-offline'),tool,tool,'test_clients.rb'])
    elif stack == 'java':
        repository = project / '.devline-offline/java/repository'
        run([tool,'-o','-B','-ntp','-Dmaven.repo.local='+str(repository),'dependency:build-classpath','-Dmdep.outputFile='+str(folder/'classpath.txt')])
        classpath = str(project/'target/classes')+':'+(folder/'classpath.txt').read_text().strip()
        run([shutil.which('java'),'-cp',classpath,'Clients'])
    elif stack == 'swift':
        run([str(temporary / 'swift-verify/build/debug/ClientProof')])
    print('PASS: '+stack+' selected service clients were added, resolved and verified with an empty HOME and outbound networking denied.',flush=True)
    print('Fixture: '+str(folder),flush=True)
finally:
    if not args.keep: shutil.rmtree(folder)
