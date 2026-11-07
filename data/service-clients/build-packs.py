#!/usr/bin/env python3
"""Refresh shared client caches using publisher package managers. Development tool."""
import json, os, pathlib, shutil, subprocess, sys, tempfile
base=pathlib.Path(__file__).resolve().parent
stack=sys.argv[1]
config=json.loads((base/(stack+'.json')).read_text())
packages={}
for entry in config['integrations']:
 for p in entry['packages']:
  if p['name'] in packages:
   packages[p['name']]['features']=sorted(set(packages[p['name']].get('features',[])+p.get('features',[])))
  else: packages[p['name']]=dict(p)
out=base/'artifacts'/stack
out.mkdir(parents=True,exist_ok=True)
(out/'ready.json').unlink(missing_ok=True)
work=pathlib.Path(tempfile.mkdtemp(prefix='devline-client-pack-'))
env=dict(os.environ,CI='true',GOTOOLCHAIN='local',GOWORK='off',DOTNET_CLI_TELEMETRY_OPTOUT='1',DOTNET_GENERATE_ASPNET_CERTIFICATE='false',DOTNET_EnableDiagnostics='0')
def run(args,cwd=work,**more): subprocess.run(args,cwd=cwd,env=dict(env,**more),check=True)
def deny(args,cwd=work,**more):
 run(['/usr/bin/sandbox-exec','-p','(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))',*args],cwd,**more)
try:
 if stack=='node':
  cache=out/'npm-cache'
  (work/'package.json').write_text(json.dumps({'name':'devline-client-pack','version':'1.0.0','private':True,'dependencies':{p['name']:p['version'] for p in packages.values()}}))
  run(['npm','install','--ignore-scripts','--no-audit','--no-fund','--cache',str(cache)])
  # Existing seeded starters must also be extensible without a registry.
  for manifest in (base.parent/'templates/node').glob('*/*/project/package-lock.json'):
   seed=work/('seed-'+manifest.parent.parent.parent.name+'-'+manifest.parent.parent.name);seed.mkdir()
   shutil.copy(manifest,seed/'package-lock.json');shutil.copy(manifest.with_name('package.json'),seed/'package.json')
   run(['npm','ci','--ignore-scripts','--no-audit','--no-fund','--cache',str(cache)],seed)
  deny(['npm','ci','--offline','--ignore-scripts','--no-audit','--no-fund','--cache',str(cache)])
  imports=';'.join('await import('+json.dumps(p['name'])+')' for p in packages.values())
  deny(['node','--input-type=module','-e',imports])
  shutil.copy(work/'package-lock.json',out/'package-lock.json')
 elif stack=='python':
  wheels=out/'wheelhouse';wheels.mkdir(exist_ok=True)
  run([sys.executable,'-m','pip','download','--disable-pip-version-check','--only-binary=:all:','--dest',str(wheels),'tomlkit',*[p['name']+'=='+p['version'] for p in packages.values()]])
  run([sys.executable,'-m','venv',str(work/'venv')]);python=str(work/'venv/bin/python')
  deny([python,'-m','pip','install','--disable-pip-version-check','--no-index','--find-links',str(wheels),*[p['name']+'=='+p['version'] for p in packages.values()]])
  deny([python,'-c','import psycopg,pymysql,pika,redis,pymongo,nats,meilisearch,boto3,authlib'])
 elif stack=='dotnet':
  cache=out/'packages'; feed=out/'feed';feed.mkdir(exist_ok=True)
  refs=''.join('<PackageReference Include="'+p['name']+'" Version="['+p['version']+']" />' for p in packages.values())
  (work/'Clients.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup><ItemGroup>'+refs+'</ItemGroup></Project>')
  (work/'Program.cs').write_text('using System;\n_ = new Npgsql.NpgsqlConnectionStringBuilder("Host=127.0.0.1"); Console.WriteLine("postgres:ok");\n_ = new MySqlConnector.MySqlConnectionStringBuilder("Server=127.0.0.1"); Console.WriteLine("mysql:ok");\n_ = new RabbitMQ.Client.ConnectionFactory(); Console.WriteLine("rabbitmq:ok");\n_ = StackExchange.Redis.ConfigurationOptions.Parse("127.0.0.1"); Console.WriteLine("redis:ok");\n_ = MongoDB.Driver.MongoClientSettings.FromConnectionString("mongodb://127.0.0.1"); Console.WriteLine("mongodb:ok");\n_ = NATS.Client.Core.NatsOpts.Default; Console.WriteLine("nats:ok");\nusing var mail = new MailKit.Net.Smtp.SmtpClient(); Console.WriteLine("mailpit:ok");\n_ = new Meilisearch.MeilisearchClient("http://127.0.0.1:7700"); Console.WriteLine("meilisearch:ok");\n_ = new Amazon.S3.AmazonS3Config { ServiceURL="http://127.0.0.1:8333" }; Console.WriteLine("seaweedfs:ok");\n_ = new Microsoft.AspNetCore.Authentication.OpenIdConnect.OpenIdConnectOptions(); Console.WriteLine("keycloak:ok");\n')
  run(['dotnet','restore','--packages' ,str(cache),'--use-lock-file','-p:NuGetAudit=false'])
  for package in cache.rglob('*.nupkg'):
   target=feed/package.name
   if not target.exists():shutil.copy(package,target)
  deny(['dotnet','restore','--packages',str(work/'fresh'),'--use-lock-file','--locked-mode','--source',str(feed),'-p:NuGetAudit=false'])
  deny(['dotnet','run','--no-restore'])
  shutil.copy(work/'packages.lock.json',out/'packages.lock.json')
 elif stack=='go':
  cache=out/'module-cache';env['GOMODCACHE']=str(cache);env['GOCACHE']=str(work/'go-build')
  run(['go','mod','init','devline.local/clientpack'])
  run(['go','get',*[p['name']+'@'+p['version'] for p in packages.values()]])
  imports=[]
  for p in packages.values():imports.extend(p.get('imports',[p['name']]))
  (work/'clients.go').write_text('package clientpack\nimport (\n'+''.join('_ '+json.dumps(name)+'\n' for name in imports)+')\n')
  run(['go','mod','tidy']);run(['go','mod','download','all'])
  for manifest in (base.parent/'templates/go').glob('*/*/project/go.mod'):
   seed=work/('seed-'+manifest.parent.parent.parent.name+'-'+manifest.parent.parent.name);seed.mkdir()
   shutil.copy(manifest,seed/'go.mod')
   if manifest.with_name('go.sum').exists():shutil.copy(manifest.with_name('go.sum'),seed/'go.sum')
   run(['go','mod','download','all'],seed)
  deny(['go','test','./...'],GOPROXY='off',GOSUMDB='off')
  shutil.copy(work/'go.mod',out/'go.mod');shutil.copy(work/'go.sum',out/'go.sum')
 elif stack=='rust':
  cache=out/'cargo-home';env['CARGO_HOME']=str(cache);env['CARGO_TARGET_DIR']=str(work/'target')
  text='[package]\nname="devline-client-pack"\nversion="0.1.0"\nedition="2024"\n[dependencies]\n'
  for p in packages.values():
   fields=['version='+json.dumps('='+p['version'])]
   if 'default_features' in p:fields.append('default-features='+str(p['default_features']).lower())
   if p.get('features'):fields.append('features='+json.dumps(p['features']))
   text+=json.dumps(p['name'])+'={'+','.join(fields)+'}\n'
  (work/'Cargo.toml').write_text(text);(work/'src').mkdir();(work/'src/main.rs').write_text('fn main() {\n let _: std::marker::PhantomData<sqlx::Postgres> = std::marker::PhantomData; println!("postgres:ok");\n let _: std::marker::PhantomData<sqlx::MySql> = std::marker::PhantomData; println!("mysql:ok");\n let _ = amqprs::connection::OpenConnectionArguments::new("127.0.0.1",5672,"devline","password"); println!("rabbitmq:ok");\n let _ = redis::Client::open("redis://127.0.0.1/").unwrap(); println!("redis:ok");\n let _ = mongodb::options::ClientOptions::default(); println!("mongodb:ok");\n let _ = async_nats::ConnectOptions::new(); println!("nats:ok");\n let _ = lettre::Message::builder(); println!("mailpit:ok");\n let _ = meilisearch_sdk::client::Client::new("http://127.0.0.1:7700",None::<String>); println!("meilisearch:ok");\n let _ = object_store::aws::AmazonS3Builder::new().with_bucket_name("devline"); println!("seaweedfs:ok");\n let _ = openidconnect::ClientId::new("devline".into()); println!("keycloak:ok");\n}\n')
  run(['cargo','fetch'])
  for manifest in (base.parent/'templates/rust').glob('*/*/project/Cargo.lock'):
   seed=work/('seed-'+manifest.parent.parent.parent.name+'-'+manifest.parent.parent.name);seed.mkdir();(seed/'src').mkdir();(seed/'src/lib.rs').write_text('')
   shutil.copy(manifest,seed/'Cargo.lock');shutil.copy(manifest.with_name('Cargo.toml'),seed/'Cargo.toml')
   if manifest.with_name('build.rs').exists():shutil.copy(manifest.with_name('build.rs'),seed/'build.rs')
   run(['cargo','fetch','--locked'],seed)
  deny(['cargo','run','--frozen'])
  shutil.copy(work/'Cargo.toml',out/'Cargo.toml');shutil.copy(work/'Cargo.lock',out/'Cargo.lock')
 else:raise SystemExit('Unsupported pack builder')
 # Keep native package archives and indexes, not duplicate extracted trees.
 if stack=='rust': shutil.rmtree(out/'cargo-home/registry/src',ignore_errors=True)
 if stack=='dotnet': shutil.rmtree(out/'packages',ignore_errors=True)
 if stack=='go':
  for item in (out/'module-cache').iterdir():
   if item.name!='cache':
    if item.is_dir():
     for child in item.rglob('*'):
      try: child.chmod(0o755 if child.is_dir() else 0o644)
      except OSError: pass
     item.chmod(0o755);shutil.rmtree(item)
    else:item.unlink()
 if stack=='node': shutil.rmtree(out/'npm-cache/_logs',ignore_errors=True)
 (out/'ready.json').write_text(json.dumps({'schema_version':1,'stack':stack,'packages':list(packages.values()),'verification':'native client graph resolved and checked with outbound networking denied'},indent=2)+'\n')
 print('READY',stack,out,flush=True)
finally:
 shutil.rmtree(work,ignore_errors=True)
