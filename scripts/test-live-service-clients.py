#!/usr/bin/env python3
"""Exercise all ten local services with the bundled Python clients.

Run through Devline's real offline project runner, for example:
  python3 scripts/test-live-service-clients.py --project /path/to/prepared-project \
    --home /path/to/library --output /path/to/results.json

The project must already have its selected Python clients and local services
prepared/running (all ten by default; use --service for an individual check). Only unique devline_probe_* resources are created and
removed; existing application data, realms, buckets, and service volumes remain.
"""
import argparse
import asyncio
from contextlib import contextmanager
from email.message import EmailMessage
from datetime import datetime, timezone
import errno
import importlib.metadata
import ipaddress
import json
import os
from pathlib import Path
import re
import secrets
import signal
import smtplib
import socket
import subprocess
import sys
import time
import traceback
from urllib.parse import unquote, urlencode, urlsplit
from urllib.request import Request, ProxyHandler, build_opener
from urllib.error import HTTPError

ROOT = Path(__file__).resolve().parents[1]
SERVICES = ['postgres', 'mysql', 'rabbitmq', 'redis', 'mongodb', 'nats', 'mailpit', 'meilisearch', 'seaweedfs', 'keycloak']
PRIMARY_CLIENTS = {'postgres':'psycopg','mysql':'PyMySQL','rabbitmq':'pika','redis':'redis','mongodb':'pymongo','nats':'nats-py','meilisearch':'meilisearch','seaweedfs':'boto3','keycloak':'Authlib'}
PROOF = 'devline_probe_' + secrets.token_hex(8)
PAYLOAD = ('Offline service proof ' + PROOF).encode()
REDACT = [value for key, value in os.environ.items() if key.startswith('DEVLINE_') and ('PASSWORD' in key or 'URL' in key)]
HTTP = build_opener(ProxyHandler({}))


def setting(name):
    value = os.environ.get(name)
    if not value:
        raise RuntimeError('Missing runner-injected setting: ' + name)
    return value


def endpoint(service):
    value = setting('DEVLINE_' + service.upper() + '_URL')
    parsed = urlsplit(value)
    if not parsed.hostname or not (parsed.hostname == 'localhost' or ipaddress.ip_address(parsed.hostname).is_loopback):
        raise RuntimeError('Probe endpoints must be local loopback services')
    return value


def scrub(value):
    result = str(value)
    for secret in sorted(REDACT, key=len, reverse=True):
        if secret:
            result = result.replace(secret, '[redacted]')
    result = re.sub(r'(://)[^/@\s]+@', r'\1[redacted]@', result)
    return re.sub(r'eyJ[A-Za-z0-9_.-]{30,}', '[redacted-token]', result)[:1400]


def http(url, *, method='GET', body=None, form=None, token=None):
    parsed = urlsplit(url)
    if parsed.scheme != 'http' or not parsed.hostname or not (parsed.hostname == 'localhost' or ipaddress.ip_address(parsed.hostname).is_loopback):
        raise RuntimeError('HTTP probe must stay on a loopback endpoint')
    headers = {}
    data = None
    if body is not None:
        headers['Content-Type'] = 'application/json'
        data = json.dumps(body).encode()
    if form is not None:
        headers['Content-Type'] = 'application/x-www-form-urlencoded'
        data = urlencode(form).encode()
    if token:
        headers['Authorization'] = 'Bearer ' + token
    try:
        with HTTP.open(Request(url, data=data, headers=headers, method=method), timeout=12) as response:
            data = response.read()
            return json.loads(data) if data and 'json' in response.headers.get('Content-Type', '') else data
    except HTTPError as error:
        detail = error.read(1000).decode(errors='replace')
        raise RuntimeError(f'HTTP {error.code} at {parsed.path}: {scrub(detail)}') from None


def network_boundary():
    try:
        connection = socket.create_connection(('1.1.1.1', 443), timeout=2)
    except OSError as error:
        assert error.errno in (errno.EPERM, errno.EACCES), 'Network failure did not prove sandbox denial'
        return ['external TCP connection rejected by the operating system']
    else:
        connection.close()
        raise RuntimeError('External networking was allowed; run this probe through devline project run')


def postgres():
    import psycopg
    from psycopg import sql
    name = sql.Identifier(PROOF)
    created = False
    def connect(): return psycopg.connect(endpoint('postgres'), connect_timeout=8, options='-c statement_timeout=10000')
    try:
        with connect() as connection:
            connection.execute(sql.SQL('CREATE TABLE {} (id integer PRIMARY KEY, body text NOT NULL)').format(name))
            connection.execute(sql.SQL('INSERT INTO {} VALUES (%s, %s)').format(name), (1, PAYLOAD.decode()))
        created = True
        with connect() as connection:
            assert connection.execute(sql.SQL('SELECT body FROM {} WHERE id=%s').format(name), (1,)).fetchone()[0] == PAYLOAD.decode()
            connection.execute(sql.SQL('UPDATE {} SET body=%s WHERE id=%s').format(name), ('updated', 1))
            assert connection.execute(sql.SQL('SELECT body FROM {} WHERE id=1').format(name)).fetchone()[0] == 'updated'
            connection.execute(sql.SQL('DELETE FROM {} WHERE id=1').format(name))
            assert connection.execute(sql.SQL('SELECT count(*) FROM {}').format(name)).fetchone()[0] == 0
    finally:
        if created:
            with connect() as connection: connection.execute(sql.SQL('DROP TABLE {}').format(name))
    return ['create table', 'committed insert', 'read through new connection', 'update/read', 'delete', 'drop own table']


def mysql():
    import pymysql
    uri = urlsplit(endpoint('mysql'))
    def connect(): return pymysql.connect(host=uri.hostname, port=uri.port, user=unquote(uri.username), password=unquote(uri.password), database=uri.path.strip('/'), connect_timeout=8, read_timeout=10, write_timeout=10, autocommit=True)
    created = False
    try:
        with connect() as connection, connection.cursor() as cursor:
            cursor.execute(f'CREATE TABLE `{PROOF}` (id integer PRIMARY KEY, body varchar(255) NOT NULL)')
            created = True
            cursor.execute(f'INSERT INTO `{PROOF}` VALUES (%s, %s)', (1, PAYLOAD.decode()))
        with connect() as connection, connection.cursor() as cursor:
            cursor.execute(f'SELECT body FROM `{PROOF}` WHERE id=%s', (1,)); assert cursor.fetchone()[0] == PAYLOAD.decode()
            cursor.execute(f'UPDATE `{PROOF}` SET body=%s WHERE id=%s', ('updated', 1))
            cursor.execute(f'SELECT body FROM `{PROOF}` WHERE id=1'); assert cursor.fetchone()[0] == 'updated'
            cursor.execute(f'DELETE FROM `{PROOF}` WHERE id=1')
            cursor.execute(f'SELECT count(*) FROM `{PROOF}`'); assert cursor.fetchone()[0] == 0
    finally:
        if created:
            with connect() as connection, connection.cursor() as cursor: cursor.execute(f'DROP TABLE `{PROOF}`')
    return ['create table', 'committed insert', 'read through new connection', 'update/read', 'delete', 'drop own table']


def rabbitmq():
    import pika
    parameters = pika.URLParameters(endpoint('rabbitmq'))
    parameters.socket_timeout = 8
    parameters.blocked_connection_timeout = 10
    parameters.connection_attempts = 1
    connection = pika.BlockingConnection(parameters)
    channel = connection.channel()
    created = False
    try:
        channel.queue_declare(queue=PROOF, durable=True)
        created = True
        channel.confirm_delivery()
        channel.basic_publish(exchange='', routing_key=PROOF, body=PAYLOAD, properties=pika.BasicProperties(delivery_mode=2), mandatory=True)
        method, _, body = channel.basic_get(PROOF, auto_ack=False)
        assert method is not None and body == PAYLOAD
        channel.basic_ack(method.delivery_tag)
        assert channel.basic_get(PROOF, auto_ack=False)[0] is None
    finally:
        if created and channel.is_open: channel.queue_delete(PROOF)
        if connection.is_open: connection.close()
    return ['declare durable queue', 'publish persistent message with broker confirmation', 'consume matching bytes', 'manual acknowledgement', 'empty queue verified', 'delete own queue']


def redis():
    import redis as redis_client
    client = redis_client.Redis.from_url(endpoint('redis'), socket_connect_timeout=8, socket_timeout=8, decode_responses=True)
    try:
        assert client.set(PROOF, '41', nx=True, ex=60)
        assert client.get(PROOF) == '41'
        assert client.incr(PROOF) == 42
        assert client.get(PROOF) == '42'
        assert client.delete(PROOF) == 1
        assert client.get(PROOF) is None
    finally:
        client.delete(PROOF)
        client.close()
    return ['set with expiry', 'read', 'atomic increment', 'read updated value', 'delete/read absent']


def mongodb():
    import pymongo
    url = endpoint('mongodb')
    def connect(): return pymongo.MongoClient(url, serverSelectionTimeoutMS=8000, connectTimeoutMS=8000, socketTimeoutMS=10000)
    created = False
    try:
        with connect() as client:
            collection = client.get_default_database()[PROOF]
            collection.insert_one({'_id': PROOF, 'body': PAYLOAD.decode()})
            created = True
        with connect() as client:
            collection = client.get_default_database()[PROOF]
            assert collection.find_one({'_id':PROOF})['body'] == PAYLOAD.decode()
            assert collection.update_one({'_id':PROOF}, {'$set':{'body':'updated'}}).modified_count == 1
            assert collection.find_one({'_id':PROOF})['body'] == 'updated'
            assert collection.delete_one({'_id':PROOF}).deleted_count == 1
            assert collection.find_one({'_id':PROOF}) is None
    finally:
        if created:
            with connect() as client: client.get_default_database().drop_collection(PROOF)
    return ['insert document', 'read through new client', 'update/read', 'delete/read absent', 'drop own collection']


async def nats_async():
    import nats as nats_client
    connection = await nats_client.connect(endpoint('nats'), connect_timeout=8, allow_reconnect=False)
    stream = PROOF.upper()
    subject = PROOF + '.stored'
    created = False
    js = connection.jetstream(timeout=10)
    try:
        async def reply(message): await message.respond(message.data + b' reply')
        await connection.subscribe(PROOF + '.request', cb=reply)
        await connection.flush()
        response = await connection.request(PROOF + '.request', PAYLOAD, timeout=5)
        assert response.data == PAYLOAD + b' reply'
        await js.add_stream(name=stream, subjects=[subject], storage='file', max_msgs=10)
        created = True
        ack = await js.publish(subject, PAYLOAD)
        assert ack.stream == stream and ack.seq == 1
        consumer = await js.pull_subscribe(subject, durable=PROOF, stream=stream)
        messages = await consumer.fetch(1, timeout=5)
        assert len(messages) == 1 and messages[0].data == PAYLOAD
        await messages[0].ack_sync(timeout=5)
        info = await js.consumer_info(stream, PROOF)
        assert info.num_ack_pending == 0
    finally:
        if created: await js.delete_stream(stream)
        await connection.drain()
    return ['request/reply', 'create file-backed JetStream stream', 'publish with persistence acknowledgement', 'durable pull consumer', 'acknowledgement confirmed', 'delete own stream']


def mailpit():
    parsed = urlsplit(endpoint('mailpit'))
    web = 'http://127.0.0.1:' + setting('DEVLINE_MAILPIT_WEB_PORT')
    message = EmailMessage()
    message['From'] = 'probe@devline.test'; message['To'] = 'inbox@devline.test'; message['Subject'] = PROOF
    message['Message-ID'] = '<' + PROOF + '@devline.test>'
    message.set_content(PAYLOAD.decode())
    identifier = None
    try:
        with smtplib.SMTP(parsed.hostname, parsed.port, timeout=10) as smtp: assert smtp.send_message(message) == {}
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            result = http(web + '/api/v1/search?' + urlencode({'query':'subject:' + PROOF}))
            matches = [row for row in result['messages'] if row['Subject'] == PROOF]
            if matches:
                identifier = matches[0]['ID']; break
            time.sleep(0.1)
        assert identifier, 'SMTP message did not reach the local HTTP inbox'
        stored = http(web + '/api/v1/message/' + identifier)
        assert PAYLOAD.decode() in stored['Text']
    finally:
        if identifier: http(web + '/api/v1/messages', method='DELETE', body={'IDs':[identifier]})
    return ['SMTP send accepted', 'search matching inbox message through HTTP API', 'read matching stored body', 'delete only own message']


def meilisearch():
    import meilisearch as search_client
    client = search_client.Client(endpoint('meilisearch'), setting('DEVLINE_MEILISEARCH_PASSWORD'), timeout=10)
    def wait(task):
        result = client.wait_for_task(task.task_uid, timeout_in_ms=30000, interval_in_ms=100)
        assert result.status == 'succeeded', 'Search task did not succeed: ' + str(result.status)
    created = False
    try:
        wait(client.create_index(PROOF, {'primaryKey':'id'}))
        created = True
        index = client.index(PROOF)
        wait(index.add_documents([{'id':PROOF, 'title':PROOF, 'body':PAYLOAD.decode()}]))
        result = index.search(PROOF)
        assert any(row['id'] == PROOF and row['body'] == PAYLOAD.decode() for row in result['hits'])
        wait(index.update_documents([{'id':PROOF, 'body':'updated'}]))
        assert index.get_document(PROOF).body == 'updated'
        wait(index.delete_document(PROOF))
        assert index.search(PROOF)['hits'] == []
    finally:
        if created: wait(client.delete_index(PROOF))
    return ['create index', 'add documents and wait for indexing', 'search matching document', 'update/read', 'delete/search absent', 'delete own index']


def seaweedfs():
    import boto3
    from botocore.config import Config
    client = boto3.client('s3', endpoint_url=endpoint('seaweedfs'), region_name='us-east-1', aws_access_key_id='devline', aws_secret_access_key=setting('DEVLINE_SEAWEEDFS_PASSWORD'), config=Config(signature_version='s3v4', s3={'addressing_style':'path'}, connect_timeout=8, read_timeout=12, retries={'max_attempts':1}))
    key = 'devline-probes/' + PROOF + '/proof.txt'
    written = False
    try:
        client.head_bucket(Bucket='devline')
        client.put_object(Bucket='devline', Key=key, Body=PAYLOAD, ContentType='text/plain')
        written = True
        obj = client.get_object(Bucket='devline', Key=key)
        assert obj['Body'].read() == PAYLOAD
        assert client.head_object(Bucket='devline', Key=key)['ContentLength'] == len(PAYLOAD)
        listed = client.list_objects_v2(Bucket='devline', Prefix='devline-probes/' + PROOF + '/')
        assert any(row['Key'] == key for row in listed.get('Contents', []))
    finally:
        if written: client.delete_object(Bucket='devline', Key=key)
        client.close()
    return ['authenticated S3 bucket access', 'SigV4 put object', 'get identical bytes', 'head object', 'list own prefix', 'delete own object']


def keycloak():
    from authlib.integrations.requests_client import OAuth2Session
    from joserfc import jwt
    from joserfc.jwk import KeySet
    from joserfc.jwt import JWTClaimsRegistry
    advertised = endpoint('keycloak')
    parsed = urlsplit(advertised)
    base = parsed.scheme + '://' + parsed.netloc
    discovery_error = None
    try:
        initial_issuer = advertised.rstrip('/') if parsed.path.strip('/') else base + '/realms/master'
        discovery = http(initial_issuer + '/.well-known/openid-configuration')
        assert discovery['issuer'] == initial_issuer, 'Initial issuer discovery mismatch'
    except Exception as error:
        discovery_error = str(error)
    admin = http(base + '/realms/master/protocol/openid-connect/token', method='POST', form={'client_id':'admin-cli', 'username':'devline', 'password':setting('DEVLINE_KEYCLOAK_PASSWORD'), 'grant_type':'password'})['access_token']
    REDACT.append(admin)
    realm = PROOF
    created = False
    client_id = PROOF + '_client'
    secret = secrets.token_hex(24)
    REDACT.append(secret)
    try:
        http(base + '/admin/realms', method='POST', body={'realm':realm,'enabled':True,'sslRequired':'none'}, token=admin)
        created = True
        http(base + '/admin/realms/' + realm + '/clients', method='POST', body={'clientId':client_id,'enabled':True,'protocol':'openid-connect','publicClient':False,'serviceAccountsEnabled':True,'standardFlowEnabled':False,'secret':secret,'protocolMappers':[{'name':'probe-resource-audience','protocol':'openid-connect','protocolMapper':'oidc-audience-mapper','consentRequired':False,'config':{'included.client.audience':client_id,'access.token.claim':'true','id.token.claim':'false','introspection.token.claim':'true'}}]}, token=admin)
        discovery = http(base + '/realms/' + realm + '/.well-known/openid-configuration')
        with OAuth2Session(client_id, secret, token_endpoint_auth_method='client_secret_basic') as session:
            session.trust_env = False
            token = session.fetch_token(discovery['token_endpoint'], grant_type='client_credentials', timeout=12)
        REDACT.append(token['access_token'])
        assert token['token_type'].lower() == 'bearer', 'Token endpoint did not issue a bearer token'
        jwks = http(discovery['jwks_uri'])
        claims = jwt.decode(token['access_token'], KeySet.import_key_set(jwks), algorithms=['RS256']).claims
        JWTClaimsRegistry(iss={'essential':True,'value':discovery['issuer']}, exp={'essential':True}).validate(claims)
        assert claims['azp'] == client_id and claims['sub'], 'JWT authorized party or subject mismatch'
        audience = claims.get('aud', [])
        assert client_id in ([audience] if isinstance(audience, str) else audience), 'JWT resource audience mismatch'
        introspection = http(discovery['introspection_endpoint'], method='POST', form={'client_id':client_id,'client_secret':secret,'token':token['access_token']})
        assert introspection['active'] is True, 'Token introspection returned inactive'
        assert introspection['client_id'] == client_id, 'Introspection client identity mismatch'
    finally:
        if created: http(base + '/admin/realms/' + realm, method='DELETE', token=admin)
    if discovery_error:
        raise RuntimeError('Real OIDC flow passed, but the advertised realm discovery is unusable: ' + discovery_error)
    return ['advertised server/admin discovery', 'disposable application issuer discovery', 'create disposable realm/client', 'Authlib client-credentials token', 'verify JWT signature/issuer/expiry/audience using JWKS', 'active token introspection', 'delete only own realm']


@contextmanager
def deadline(seconds):
    previous = signal.signal(signal.SIGALRM, lambda *_: (_ for _ in ()).throw(TimeoutError('Service probe deadline exceeded')))
    signal.alarm(seconds)
    try: yield
    finally: signal.alarm(0); signal.signal(signal.SIGALRM, previous)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', type=Path)
    parser.add_argument('--home', type=Path)
    parser.add_argument('--cli', type=Path, default=ROOT/'core/target/debug/devline')
    parser.add_argument('--catalog', type=Path, default=ROOT/'data')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--service', action='append', choices=SERVICES)
    parser.add_argument('--assert-isolated', action='store_true', help='Require one selected service and prove all other primary clients are absent.')
    parser.add_argument('--probe', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not args.probe:
        if args.project is None or args.home is None: parser.error('--project and --home are required')
        command = [str(args.cli.resolve()),'--offline','--home',str(args.home.resolve()),'--catalog',str(args.catalog.resolve()),'project','--path',str(args.project.resolve()),'run','--','python3',str(Path(__file__).resolve()),'--probe','--output',str(args.output.resolve())]
        for service in args.service or []: command += ['--service',service]
        if args.assert_isolated: command += ['--assert-isolated']
        return subprocess.call(command)
    # Refuse every live operation unless the actual runner boundary is active.
    boundary = network_boundary()
    selected = args.service or SERVICES
    installed = {distribution.metadata['Name']:distribution.version for distribution in importlib.metadata.distributions()}
    if args.assert_isolated:
        if len(selected) != 1: parser.error('--assert-isolated requires exactly one --service')
        for service, package in PRIMARY_CLIENTS.items():
            if service in selected: continue
            try: importlib.metadata.version(package)
            except importlib.metadata.PackageNotFoundError: pass
            else: raise RuntimeError('Unselected primary client is installed: ' + package)
    results = []
    for service in selected:
        started = time.monotonic()
        try:
            with deadline(75):
                operations = asyncio.run(nats_async()) if service == 'nats' else globals()[service]()
            result = {'service':service,'status':'passed','operations':operations,'cleanup':'completed'}
        except Exception as error:
            location = traceback.extract_tb(error.__traceback__)[-1]
            result = {'service':service,'status':'failed','error':type(error).__name__ + ': ' + scrub(error), 'step':location.name + ':' + str(location.lineno)}
        result['seconds'] = round(time.monotonic()-started, 2)
        results.append(result)
        print(result['status'].upper() + ' ' + service + (': ' + result['error'] if 'error' in result else ''), flush=True)
        report = {'schema_version':1,'measured_at':datetime.now(timezone.utc).isoformat(),'scope':'Live protocol checks using prepared Python clients; other runtimes are tested separately.','namespace':PROOF,'runtime':'python','network_boundary':boundary,'selected_services':selected,'unselected_primary_clients_absent':args.assert_isolated,'packages':dict(sorted(installed.items())),'results':results}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report,indent=2)+'\n')
    return 0 if all(row['status']=='passed' for row in results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
