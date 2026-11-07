#!/usr/bin/env python3
"""Seal prepared client packs after their offline proof, or validate existing seals."""
import argparse
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]/'data/service-clients/artifacts'
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');p.add_argument('stacks',nargs='*');args=p.parse_args()
for root in sorted(ROOT.iterdir()):
 if not root.is_dir() or args.stacks and root.name not in args.stacks:continue
 ready=root/'ready.json'
 if not ready.is_file():raise SystemExit('No completed proof marker: '+str(root))
 records=[]
 for file in sorted(root.rglob('*')):
  if file.is_symlink():raise SystemExit('Symlink in client pack: '+str(file))
  if not file.is_file() or file in (ready,root/'inventory.json'):continue
  records.append({'path':file.relative_to(root).as_posix(),'bytes':file.stat().st_size,'sha256':hashlib.file_digest(file.open('rb'),'sha256').hexdigest()})
 content=(json.dumps({'schema_version':1,'stack':root.name,'files':records},indent=2)+'\n').encode()
 digest=hashlib.sha256(content).hexdigest();metadata=json.loads(ready.read_text())
 if args.check:
  assert (root/'inventory.json').read_bytes()==content and metadata.get('inventory_sha256')==digest, root.name+' seal changed'
 else:
  (root/'inventory.json').write_bytes(content);metadata['inventory_sha256']=digest;ready.write_text(json.dumps(metadata,indent=2)+'\n')
 print(('Verified ' if args.check else 'Sealed ')+root.name+': '+str(len(records))+' files')
