#!/usr/bin/env python3
"""Prove opt-in client additions through the CLI, then execute each runtime offline.

One project per runtime exercises all ten service capabilities without contacting
a server. This tests package availability and public client APIs, not live server
compatibility. The separate .NET service journey covers real database/broker I/O.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]
STACKS = ["dotnet", "elixir", "go", "java", "node", "python", "ruby", "rust", "swift"]
POLICY = '(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))'
PREFIX = ["/usr/bin/sandbox-exec", "-p", POLICY]
MARKER = "DEVLINE_PROBE\t"

DOTNET = r'''using System;
using Amazon.Runtime;
using Amazon.S3;
using Amazon.S3.Model;
static void Pass(string id, bool value) { if (!value) throw new Exception(id); Console.WriteLine("DEVLINE_PROBE\t" + id); }
Pass("postgres", new Npgsql.NpgsqlConnectionStringBuilder("Host=localhost;Database=devline").Database == "devline");
Pass("mysql", new MySqlConnector.MySqlConnectionStringBuilder("Server=localhost;Database=devline").Database == "devline");
Pass("rabbitmq", new RabbitMQ.Client.ConnectionFactory { Uri = new Uri("amqp://guest:guest@localhost/devline") }.VirtualHost == "devline");
Pass("redis", StackExchange.Redis.ConfigurationOptions.Parse("127.0.0.1:6379").EndPoints.Count == 1);
Pass("mongodb", MongoDB.Driver.MongoClientSettings.FromConnectionString("mongodb://127.0.0.1:27017").Server.Port == 27017);
Pass("nats", NATS.Client.Core.NatsOpts.Default.Url.StartsWith("nats://"));
using (var smtp = new MailKit.Net.Smtp.SmtpClient()) Pass("mailpit", !smtp.IsConnected);
var search = new Meilisearch.MeilisearchClient("http://127.0.0.1:7700", "offline");
Pass("meilisearch", search.Index("notes") != null);
using var s3 = new AmazonS3Client(new BasicAWSCredentials("offline", "offline-secret"), new AmazonS3Config { ServiceURL = "http://127.0.0.1:8333", AuthenticationRegion = "us-east-1", ForcePathStyle = true });
var signed = await s3.GetPreSignedURLAsync(new GetPreSignedUrlRequest { BucketName = "notes", Key = "proof", Expires = DateTime.UtcNow.AddMinutes(5) });
Pass("seaweedfs", signed.Contains("X-Amz-Signature="));
var oidc = new Microsoft.AspNetCore.Authentication.OpenIdConnect.OpenIdConnectOptions { Authority = "http://127.0.0.1:8080/realms/devline", ClientId = "offline" };
Pass("keycloak", oidc.ClientId == "offline");
'''

PYTHON = r'''import smtplib
import psycopg.conninfo, pymysql, pika, redis, pymongo, nats, meilisearch, boto3
from authlib.jose import jwt
def passed(name, value):
    assert value, name
    print("DEVLINE_PROBE\t" + name)
passed("postgres", psycopg.conninfo.conninfo_to_dict("host=localhost dbname=devline")["dbname"] == "devline")
passed("mysql", pymysql.converters.escape_string("a'b") == "a\\'b")
passed("rabbitmq", pika.URLParameters("amqp://guest:guest@localhost/devline").virtual_host == "devline")
passed("redis", redis.Redis.from_url("redis://127.0.0.1:6379/0").connection_pool.connection_kwargs["host"] == "127.0.0.1")
with pymongo.MongoClient("mongodb://127.0.0.1:27017", connect=False) as client:
    passed("mongodb", client.get_database("devline").name == "devline")
passed("nats", not nats.NATS().is_connected)
passed("mailpit", smtplib.SMTP().sock is None)
passed("meilisearch", meilisearch.Client("http://127.0.0.1:7700", "offline").index("notes").uid == "notes")
client = boto3.client("s3", endpoint_url="http://127.0.0.1:8333", region_name="us-east-1", aws_access_key_id="offline", aws_secret_access_key="offline-secret")
passed("seaweedfs", "proof" in client.generate_presigned_url("get_object", Params={"Bucket":"notes","Key":"proof"}, ExpiresIn=60))
key = b"0123456789abcdef0123456789abcdef"
token = jwt.encode({"alg":"HS256"}, {"sub":"offline"}, key)
passed("keycloak", jwt.decode(token, key)["sub"] == "offline")
'''

NODE = r'''import assert from 'node:assert/strict';
import pg from 'pg'; import mysql from 'mysql2'; import amqp from 'amqplib';
import {createClient} from 'redis'; import {MongoClient} from 'mongodb';
import * as nats from '@nats-io/transport-node'; import nodemailer from 'nodemailer';
import {Meilisearch} from 'meilisearch'; import {S3Client, GetObjectCommand} from '@aws-sdk/client-s3';
import * as oidc from 'openid-client';
function passed(id, value) { assert.ok(value, id); console.log('DEVLINE_PROBE\t'+id); }
passed('postgres', new pg.Client({connectionString:'postgresql://user:pass@127.0.0.1/devline'}).database === 'devline');
passed('mysql', mysql.escape("a'b").includes("\\'"));
passed('rabbitmq', amqp.credentials.plain('guest','guest').mechanism === 'PLAIN');
passed('redis', !createClient({url:'redis://127.0.0.1:6379'}).isOpen);
const mongo = new MongoClient('mongodb://127.0.0.1:27017');
passed('mongodb', mongo.db('devline').databaseName === 'devline'); await mongo.close();
passed('nats', typeof nats.connect === 'function');
const mail = nodemailer.createTransport({streamTransport:true});
const message = await mail.sendMail({from:'sender@example.test',to:'receiver@example.test',subject:'offline',text:'proof'});
passed('mailpit', Boolean(message.messageId)); mail.close();
passed('meilisearch', new Meilisearch({host:'http://127.0.0.1:7700'}).index('notes').uid === 'notes');
const s3 = new S3Client({endpoint:'http://127.0.0.1:8333',region:'us-east-1',credentials:{accessKeyId:'offline',secretAccessKey:'offline-secret'}});
passed('seaweedfs', new GetObjectCommand({Bucket:'notes',Key:'proof'}).input.Key === 'proof'); s3.destroy();
passed('keycloak', (await oidc.calculatePKCECodeChallenge(oidc.randomPKCECodeVerifier())).length > 20);
'''

GO = r'''package main
import (
 "fmt"
 "net/smtp"
 "github.com/jackc/pgx/v5"
 "github.com/go-sql-driver/mysql"
 amqp "github.com/rabbitmq/amqp091-go"
 "github.com/redis/go-redis/v9"
 "go.mongodb.org/mongo-driver/v2/mongo/options"
 "github.com/nats-io/nats.go"
 "github.com/meilisearch/meilisearch-go"
 "github.com/aws/aws-sdk-go-v2/service/s3"
 "github.com/aws/aws-sdk-go-v2/credentials"
 "github.com/coreos/go-oidc/v3/oidc"
)
func pass(id string, value bool) { if !value { panic(id) }; fmt.Println("DEVLINE_PROBE\t"+id) }
func main() {
 pg, err := pgx.ParseConfig("postgres://user:pass@127.0.0.1/devline"); pass("postgres", err==nil && pg.Database=="devline")
 my, err := mysql.ParseDSN("user:pass@tcp(127.0.0.1:3306)/devline"); pass("mysql", err==nil && my.DBName=="devline")
 rabbit, err := amqp.ParseURI("amqp://guest:guest@127.0.0.1/devline"); pass("rabbitmq", err==nil && rabbit.Vhost=="devline")
 rd, err := redis.ParseURL("redis://127.0.0.1:6379/0"); pass("redis", err==nil && rd.Addr=="127.0.0.1:6379")
 pass("mongodb", options.Client().ApplyURI("mongodb://127.0.0.1:27017").Validate()==nil)
 nat := nats.GetDefaultOptions(); nat.Url = "nats://127.0.0.1:4222"; pass("nats", nat.Url=="nats://127.0.0.1:4222")
 pass("mailpit", smtp.PlainAuth("", "offline", "secret", "localhost")!=nil)
 pass("meilisearch", meilisearch.New("http://127.0.0.1:7700", meilisearch.WithAPIKey("offline"))!=nil)
 endpoint := "http://127.0.0.1:8333"
 pass("seaweedfs", s3.New(s3.Options{Region:"us-east-1", BaseEndpoint:&endpoint, Credentials:credentials.NewStaticCredentialsProvider("offline","secret", "")})!=nil)
 pass("keycloak", (&oidc.Config{ClientID:"offline"}).ClientID=="offline")
}
'''

RUST = r'''fn pass(id: &str) { println!("DEVLINE_PROBE\t{id}"); }
fn main() {
 let _ = sqlx::postgres::PgConnectOptions::new().host("127.0.0.1").database("devline"); pass("postgres");
 let _ = sqlx::mysql::MySqlConnectOptions::new().host("127.0.0.1").database("devline"); pass("mysql");
 let _ = amqprs::connection::OpenConnectionArguments::new("127.0.0.1", 5672, "guest", "guest"); pass("rabbitmq");
 let _ = redis::Client::open("redis://127.0.0.1:6379/0").unwrap(); pass("redis");
 let _ = mongodb::options::ClientOptions::builder().app_name("devline".to_owned()).build(); pass("mongodb");
 let _ = async_nats::ConnectOptions::new().name("devline"); pass("nats");
 let _: lettre::Address = "sender@example.test".parse().unwrap(); pass("mailpit");
 let _ = meilisearch_sdk::client::Client::new("http://127.0.0.1:7700", Some("offline")).unwrap(); pass("meilisearch");
 let _ = object_store::aws::AmazonS3Builder::new().with_endpoint("http://127.0.0.1:8333").with_bucket_name("notes"); pass("seaweedfs");
 let issuer = openidconnect::IssuerUrl::new("http://127.0.0.1:8080/realms/devline".to_owned()).unwrap(); assert!(issuer.as_str().contains("devline")); pass("keycloak");
}
'''


def write(project, name, value):
    path = project / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value)


def fixture(stack, project):
    project.mkdir()
    if stack == "dotnet":
        write(project, "Clients.csproj", '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
        write(project, "Program.cs", DOTNET)
        return ["dotnet", "run", "-p:UseSharedCompilation=false"]
    if stack == "python":
        write(project, "pyproject.toml", '[project]\nname="client-proof"\nversion="0.1.0"\ndependencies=[]\n')
        write(project, "probe.py", PYTHON)
        return ["python3", "probe.py"]
    if stack == "node":
        package = {"name":"client-proof", "version":"0.1.0", "private":True, "type":"module", "scripts":{"test":"node probe.mjs"}}
        write(project, "package.json", json.dumps(package))
        write(project, "package-lock.json", json.dumps({"name":"client-proof", "version":"0.1.0", "lockfileVersion":3, "requires":True, "packages":{"":{"name":"client-proof","version":"0.1.0"}}}))
        write(project, "probe.mjs", NODE)
        return ["node", "probe.mjs"]
    if stack == "go":
        write(project, "go.mod", "module devline.local/clientproof\n\ngo 1.27.0\n")
        write(project, "main.go", GO)
        return ["go", "run", "-mod=vendor", "."]
    if stack == "rust":
        write(project, "Cargo.toml", '[package]\nname="client-proof"\nversion="0.1.0"\nedition="2024"\n[dependencies]\n')
        write(project, "src/main.rs", RUST)
        return ["cargo", "run", "--frozen", "--quiet"]
    if stack == "ruby":
        write(project, "Gemfile", 'source "https://rubygems.org"\n')
        entries = [("postgres","pg","PG::Connection"),("mysql","trilogy","Trilogy"),("rabbitmq","bunny","Bunny::Session"),("redis","redis","Redis"),("mongodb","mongo","Mongo::Client"),("nats","nats/client","NATS::IO::Client"),("mailpit","net/smtp","Net::SMTP"),("meilisearch","meilisearch","MeiliSearch::Client"),("seaweedfs","aws-sdk-s3","Aws::S3::Client"),("keycloak","openid_connect","OpenIDConnect::Client")]
        code = 'require "bundler/setup"\n'
        for service, module, symbol in entries:
            code += f'require {json.dumps(module)}\nraise {json.dumps(service)} unless {symbol}.is_a?(Class)\nputs "DEVLINE_PROBE\\t{service}"\n'
        write(project, "probe.rb", code)
        return ["ruby", "probe.rb"]
    if stack == "java":
        write(project, "pom.xml", '<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion><groupId>dev.devline</groupId><artifactId>client-proof</artifactId><version>1.0.0</version><properties><maven.compiler.release>26</maven.compiler.release><project.build.sourceEncoding>UTF-8</project.build.sourceEncoding></properties><build><plugins><plugin><groupId>org.apache.maven.plugins</groupId><artifactId>maven-compiler-plugin</artifactId><version>3.14.1</version></plugin></plugins></build></project>')
        entries = [("postgres","org.postgresql.Driver"),("mysql","com.mysql.cj.jdbc.Driver"),("rabbitmq","com.rabbitmq.client.ConnectionFactory"),("redis","io.lettuce.core.RedisClient"),("mongodb","com.mongodb.client.MongoClients"),("nats","io.nats.client.Nats"),("mailpit","jakarta.mail.Session"),("meilisearch","com.meilisearch.sdk.Client"),("seaweedfs","software.amazon.awssdk.services.s3.S3Client"),("keycloak","com.nimbusds.oauth2.sdk.AuthorizationRequest")]
        code = 'public class Clients { public static void main(String[] args) throws Exception {\n'
        for service, symbol in entries:
            code += f'if (Class.forName({json.dumps(symbol)}).getMethods().length == 0) throw new Exception({json.dumps(service)}); System.out.println("DEVLINE_PROBE\\t{service}");\n'
        write(project, "src/main/java/Clients.java", code + "}}\n")
        write(project, "probe.sh", 'set -eu\nmvn -o -B -ntp compile dependency:build-classpath -Dmdep.outputFile=.devline-probe-classpath\nexec java -cp "target/classes:$(cat .devline-probe-classpath)" Clients\n')
        return ["/bin/sh", "probe.sh"]
    if stack == "elixir":
        write(project, "mix.exs", 'defmodule ClientProof.MixProject do\n use Mix.Project\n def project, do: [app: :client_proof, version: "0.1.0", deps: []]\n def application, do: [extra_applications: [:logger]]\nend\n')
        entries = [("postgres","Postgrex"),("mysql","MyXQL"),("rabbitmq","AMQP.Connection"),("redis","Redix"),("mongodb","Mongo"),("nats","Gnat"),("mailpit",":gen_smtp_client"),("meilisearch","Req"),("seaweedfs","ExAws.S3"),("keycloak",":oidcc")]
        code = ""
        for service, symbol in entries:
            code += f'unless Code.ensure_loaded?({symbol}) and length({symbol}.module_info(:exports)) > 0, do: raise("{service}")\nIO.puts("DEVLINE_PROBE\\t{service}")\n'
        write(project, "probe.exs", code)
        write(project, "test/test_helper.exs", "ExUnit.start()\n")
        write(project, "test/clients_test.exs", 'defmodule ClientProofTest do\n use ExUnit.Case\n test "public clients load" do\n Code.require_file("probe.exs")\n end\nend\n')
        return ["elixir", "-S", "mix", "run", "probe.exs"]
    if stack == "swift":
        write(project, "Package.swift", '// swift-tools-version: 6.1\nimport PackageDescription\nlet package = Package(name: "ClientProof", platforms: [.macOS(.v14)], targets: [.executableTarget(name: "ClientProof")])\n')
        entries = [("postgres","PostgresNIO","PostgresConnection"),("mysql","MySQLNIO","MySQLConnection"),("rabbitmq","BunnySwift","BunnySwift.Connection"),("redis","RediStack","RedisConnection"),("mongodb","MongoKitten","MongoDatabase"),("nats","Nats","NatsClient"),("meilisearch","MeiliSearch","MeiliSearch"),("seaweedfs","SotoS3","S3"),("keycloak","AppAuth","OIDServiceConfiguration")]
        code = "import Foundation\n"
        for _, module, _ in entries: code += "import " + module + "\n"
        for service, module, symbol in entries:
            code += f'precondition(!String(reflecting: {symbol}.self).isEmpty)\nprint("DEVLINE_PROBE\\t{service}")\n'
        code += 'precondition(URLRequest(url: URL(string: "http://127.0.0.1:8025/api/v1/messages")!).httpMethod == "GET")\nprint("DEVLINE_PROBE\\tmailpit")\n'
        write(project, "Sources/ClientProof/main.swift", code)
        return ["swift", "run", "-j", "4", "ClientProof"]
    raise AssertionError(stack)


def fingerprint(project):
    digest = hashlib.sha256()
    for path in sorted(project.rglob("*")):
        if path.is_symlink():
            digest.update(str(path.relative_to(project)).encode())
            digest.update(os.readlink(path).encode())
            continue
        if path.is_file():
            digest.update(str(path.relative_to(project)).encode())
            digest.update(path.read_bytes())
    return digest.hexdigest()


def environment(home):
    env = {"PATH":os.environ.get("PATH", "/usr/bin:/bin"), "HOME":str(home), "LANG":"en_US.UTF-8", "CI":"true", "GIT_CONFIG_NOSYSTEM":"1", "GIT_CONFIG_GLOBAL":"/dev/null", "GIT_TERMINAL_PROMPT":"0", "PIP_CONFIG_FILE":"/dev/null", "PIP_DISABLE_PIP_VERSION_CHECK":"1", "NPM_CONFIG_USERCONFIG":"/dev/null", "AWS_EC2_METADATA_DISABLED":"true", "DOTNET_CLI_TELEMETRY_OPTOUT":"1", "DOTNET_EnableDiagnostics":"0", "DOTNET_CLI_USE_MSBUILD_SERVER":"0", "MSBUILDDISABLENODEREUSE":"1"}
    for name in ["TMPDIR", "XDG_CACHE_HOME", "CARGO_HOME", "CARGO_TARGET_DIR", "GOCACHE", "GOMODCACHE", "NUGET_PACKAGES", "NUGET_HTTP_CACHE_PATH", "DOTNET_CLI_HOME", "BUNDLE_USER_HOME", "MIX_HOME", "HEX_HOME", "GRADLE_USER_HOME", "npm_config_cache"]:
        folder = home / name.lower(); folder.mkdir(parents=True)
        env[name] = str(folder)
    for name in ["JAVA_HOME", "DEVELOPER_DIR", "TOOLCHAINS", "DEVLINE_OPENSSL_DIR"]:
        if name in os.environ: env[name] = os.environ[name]
    for executable in ["dotnet","elixir","go","mvn","node","python3","ruby","cargo","swift"]:
        tool = shutil.which(executable)
        if tool:
            if executable == "cargo" and shutil.which("rustup"):
                located = subprocess.run(["rustup","which","cargo"],capture_output=True,text=True)
                if located.returncode == 0: tool = located.stdout.strip()
            if executable == "swift":
                located = subprocess.run(["/usr/bin/xcrun","--find","swift"],capture_output=True,text=True)
                if located.returncode == 0: tool = located.stdout.strip()
            env["DEVLINE_" + executable.upper()] = tool
    return env


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stack", action="append", choices=STACKS, help="Repeat for selected runtimes; default all nine.")
    parser.add_argument("--cli", type=Path, default=ROOT / "core/target/debug/devline")
    parser.add_argument("--output", type=Path, help="Keep fixtures, logs, and structured results here.")
    parser.add_argument("--timeout", type=int, default=900, help="Maximum seconds for each native operation.")
    parser.add_argument("--guards-only", action="store_true", help="Check opt-in and rollback behavior without rebuilding all client probes.")
    args = parser.parse_args()
    cli = args.cli.resolve()
    if not cli.is_file() or not Path(PREFIX[0]).is_file(): parser.error("Build devline and run this proof on macOS.")
    output = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="devline-client-journeys-"))
    output.mkdir(parents=True, exist_ok=True)
    results, checks = [], []
    print("Evidence: " + str(output), flush=True)

    for stack in args.stack or STACKS:
        directory = output / stack
        if directory.exists(): raise SystemExit("Use a fresh output directory: " + str(directory))
        directory.mkdir()
        project = directory / "project"
        run_args = fixture(stack, project)
        env = environment(directory / "empty-home")
        base = [str(cli), "--offline", "--home", str(directory / "library"), "--catalog", str(ROOT / "data")]
        step = 0

        def run(command, label, success=True, parse=False):
            nonlocal step
            step += 1
            logfile = directory / f"{step:02d}-{label}.log"
            start = time.monotonic()
            with logfile.open("wb") as log:
                # macOS cannot nest sandbox profiles. The CLI applies its own
                # mutation/run profiles; test that real boundary after prepare.
                launched = command if command[0] == str(cli) else PREFIX + command
                process = subprocess.Popen(launched, cwd=project, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                try: code = process.wait(timeout=args.timeout)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL); process.wait()
                    raise RuntimeError(f"{stack} {label} timed out; {logfile}")
            text = logfile.read_text(errors="replace")
            checks.append({"stack":stack,"step":label,"command":command,"exit_code":code,"seconds":round(time.monotonic()-start,2),"log":str(logfile)})
            if (code == 0) != success: raise RuntimeError(f"{stack} {label}: exit {code}; {logfile}\n" + text[-5000:])
            if parse:
                try:
                    value = json.loads(text)
                    return value.get("data", value)
                except ValueError:
                    pass
                for line in reversed(text.splitlines()):
                    try:
                        value = json.loads(line)
                        if value.get("ok") is True: return value["data"]
                    except (ValueError, AttributeError): pass
                raise RuntimeError("No successful CLI JSON result in " + str(logfile))
            return text

        try:
            # Prove the boundary itself, not merely the --offline option.
            probe = 'import errno,socket\ntry: socket.create_connection(("1.1.1.1",443),timeout=2)\nexcept OSError as e: assert e.errno in (errno.EPERM,errno.EACCES),e\nelse: raise AssertionError("outbound networking allowed")'
            run([sys.executable,"-c",probe], "network-boundary")
            initial = fingerprint(project)
            run(base + ["infra","add",str(project)], "no-selection", success=False)
            assert fingerprint(project) == initial, "Missing selections changed the project"
            run(base + ["infra","add",str(project),"--client",stack+":postgres","--client","invalid:client"], "invalid-selection", success=False)
            assert fingerprint(project) == initial, "Invalid selections changed the project"
            source_files = {path.relative_to(project): path.read_bytes() for path in project.rglob("*") if path.is_file()}
            postgres = next(item for item in json.loads((ROOT/"data/infrastructure.json").read_text())["services"] if item["id"] == "postgres")
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0))
                port = reservation.getsockname()[1]
            run(base + ["--json","infra","add",str(project),"--service","postgres@"+postgres["version"],"--port",f"postgres:database={port}"], "image-only-no-clients", parse=True)
            assert all((project/name).read_bytes() == contents for name, contents in source_files.items()), "Choosing an image changed source or dependency declarations"
            if stack == "python":
                failed_project = directory / "staged-failure"
                fixture("python", failed_project)
                write(failed_project, "probe.py", "def invalid_syntax(:\n")
                before_failure = fingerprint(failed_project)
                failure = run(base + ["infra","add",str(failed_project),"--client","python:rabbitmq"], "staged-failure-preserves-original", success=False)
                assert "SyntaxError" in failure, "Failure did not reach actual staged project verification"
                assert fingerprint(failed_project) == before_failure, "Failed verification changed original files"
            if args.guards_only:
                results.append({"runtime":stack,"state":"passed","scope":"mutation_guards"})
                print(f"PASS {stack}: mutation guards", flush=True)
                continue
            definitions = run(base + ["--json","infra","clients",str(project)], "suggestions", parse=True)
            supported = [library for service in definitions["services"] for library in service["libraries"] if library["supported"] and not library["builtin"]]
            selections = sum((["--client", item["id"]] for item in supported), [])
            added = run(base + ["--json","infra","add",str(project)] + selections, "add-clients", parse=True)
            assert added.get("ready_offline") is True and added.get("changed") is True, added
            run(base + ["project","--path",str(project),"run","--",sys.executable,"-c",probe], "project-network-boundary")
            text = run(base + ["project","--path",str(project),"run","--"] + run_args, "execute-client-apis")
            observed = {line.split("\t",1)[1].strip() for line in text.splitlines() if line.startswith(MARKER)}
            expected = {service["id"] for service in definitions["services"]}
            assert observed == expected, f"Missing/extra service probes: {expected ^ observed}"
            before_repeat = fingerprint(project)
            repeated = run(base + ["--json","infra","add",str(project)] + selections, "repeat-selection", parse=True)
            assert repeated.get("changed") is False and fingerprint(project) == before_repeat, "Repeating clients must preserve the project"
            for service in definitions["services"]:
                results.append({"runtime":stack,"service":service["id"],"state":"passed","evidence":"CLI dependency transaction and executed public client API with outbound network denied","builtin":all(item["builtin"] for item in service["libraries"])})
            print(f"PASS {stack}: {len(observed)} service capabilities", flush=True)
        except Exception as error:
            results.append({"runtime":stack,"state":"failed","detail":str(error)})
            print(f"FAIL {stack}: {error}", flush=True)
        finally:
            (output / "results.json").write_text(json.dumps({"results":results,"operations":checks},indent=2)+"\n")
    failures = [result for result in results if result["state"] != "passed"]
    scope = "runtime mutation guards" if args.guards_only else "service capabilities"
    print(f"{len(results)-len(failures)} {scope} passed; {len(failures)} runtime failures. Evidence: {output}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
