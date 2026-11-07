#!/usr/bin/env python3
"""Exercise the running app over loopback. No Python packages or network downloads."""
import argparse
import json
import time
import urllib.error
import urllib.request
import uuid

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url', default='http://127.0.0.1:5080')
parser.add_argument('--verify-note', help='Verify this note still exists and is processed after restarting the app/services.')
args = parser.parse_args()

def request(path, body=None):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(args.url.rstrip('/') + path, data=data, headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(req, timeout=5) as response:
        return json.load(response)

health = request('/health/ready')
assert health['ready'], health
assert health['database'] == {'provider': 'postgresql', 'ready': True}, health
assert health['queue']['configured'] and health['queue']['ready'], health
if args.verify_note:
    note_id = args.verify_note
else:
    text = 'Offline proof ' + str(uuid.uuid4())
    note = request('/api/notes', {'text': text})
    assert note['text'] == text and note['queueRequested'], note
    note_id = note['id']
for attempt in range(60):
    note = request('/api/notes/' + note_id)
    if note['processedAtUtc'] and note['publishedAtUtc']:
        assert note['delivery'] == 'processed', note
        print('PASS: PostgreSQL saved the note; RabbitMQ confirmed publication and the worker processed it.')
        print('Note ID: ' + note_id)
        print('After restarting the app and services, verify persistence with:')
        print('python3 scripts/test-services.py --url ' + args.url + ' --verify-note ' + note_id)
        break
    time.sleep(0.5)
else:
    raise SystemExit('FAIL: note remained pending: ' + json.dumps(note))
