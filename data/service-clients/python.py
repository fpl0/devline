"""Add explicitly chosen Python clients in Devline's disposable project copy."""
import hashlib, json, os, pathlib, re, subprocess, sys, tempfile, zipfile
from email.parser import BytesParser
project=pathlib.Path(sys.argv[1]); selection=json.loads(pathlib.Path(sys.argv[2]).read_text()); offline=sys.argv[3]=='true'; tool=sys.argv[4]
base=pathlib.Path(selection['catalog_dir']); artifacts=base/'service-clients/artifacts/python/wheelhouse'
managed_relative=os.environ.get('DEVLINE_PROJECT_CACHE','.devline-offline')
cache=project/managed_relative;cache.mkdir(exist_ok=True,parents=True)
bundled=project/'.devline/packages' if managed_relative=='.devline/cache' else project
scratch=pathlib.Path(tempfile.mkdtemp(dir=os.environ['TMPDIR'],prefix='python-clients-'))
def run(args):subprocess.run(args,cwd=project,check=True)
run([tool,'-I','-m','venv',str(scratch/'venv')]);python=str(scratch/'venv/bin/python')
pip=[python,'-I','-m','pip','--isolated','--disable-pip-version-check','--no-cache-dir']
selected=[p['name']+'=='+p['version'] for row in selection['integrations'] for p in row['packages']]
find=[]
for path in [artifacts,bundled/'wheelhouse',cache/'wheelhouse']:
 if path.is_dir():find+=['--find-links',str(path)]
# Reuse bundled starter wheels for unchanged application dependencies.
for path in (base/'templates/python').glob('*/*/project/wheelhouse'):find+=['--find-links',str(path)]
requirements=[]
primary=next((project/name for name in ['requirements.lock','requirements.txt','requirements.in'] if (project/name).is_file()),None)
if primary:
 for line in primary.read_text().replace('\\\n',' ').splitlines():
  line=re.sub(r'\s+--hash(?:=|\s)\S+','',line).strip()
  if line:requirements.append(line)
else:
 manifest=project/'pyproject.toml'
 if not manifest.is_file():raise SystemExit('Choose a Python project with a requirements file or static pyproject.toml dependencies.')
 run(pip+['install','--no-index','--find-links',str(artifacts),'tomlkit'])
 edit=scratch/'edit.py'
 edit.write_text('''import json,sys,tomlkit\nfrom pathlib import Path\np=Path(sys.argv[1]); d=tomlkit.parse(p.read_text()); section=d.get("project")\nif section is None or "dependencies" in section.get("dynamic",[]): raise SystemExit("Computed Python dependencies cannot be changed safely; choose a static project manifest.")\nnew=json.loads(sys.argv[2]); deps=section.get("dependencies",tomlkit.array()); deps.extend(new); section["dependencies"]=deps\np.write_text(tomlkit.dumps(d))\nrequirements=list(deps)\nfor group in ("dev","test"):\n requirements+=list(section.get("optional-dependencies",{}).get(group,[]))\n requirements+=list(d.get("dependency-groups",{}).get(group,[]))\nPath(sys.argv[3]).write_text("\\n".join(requirements)+"\\n")\n''')
 exported=scratch/'requirements.txt'
 run([python,'-I',str(edit),str(manifest),json.dumps(selected),str(exported)])
 requirements=exported.read_text().splitlines()
 if (project/'uv.lock').exists():
  args=['uv','lock','--no-cache']
  if offline:args+=['--offline','--no-index',*find]
  run(args)
 elif (project/'poetry.lock').exists() or (project/'Pipfile.lock').exists():
  raise SystemExit('This project uses a different lock manager; no original files were changed.')
requirements+=selected
source=scratch/'selected-requirements.txt';source.write_text('\n'.join(dict.fromkeys(requirements))+'\n')
wheels=scratch/'resolved-wheels';wheels.mkdir()
args=pip+['download','--only-binary=:all:','--dest',str(wheels),*find,'-r',str(source)]
if offline:args+=['--no-index']
run(args)
lock=[]
for wheel in sorted(wheels.glob('*.whl')):
 with zipfile.ZipFile(wheel) as archive:
  meta=BytesParser().parsebytes(archive.read(next(name for name in archive.namelist() if name.endswith('.dist-info/METADATA'))),headersonly=True)
 lock.append(meta['Name']+'=='+meta['Version']+' --hash=sha256:'+hashlib.sha256(wheel.read_bytes()).hexdigest())
text='\n'.join(lock)+'\n'
import shutil
if (cache/'wheelhouse').exists():shutil.rmtree(cache/'wheelhouse')
shutil.move(str(wheels),cache/'wheelhouse');(cache/'requirements.lock').write_text(text)
if primary:
 primary.write_text(text)
 if primary.name!='requirements.in' and (project/'requirements.in').is_file():
  with (project/'requirements.in').open('a') as f:f.write('\n'+'\n'.join(selected)+'\n')
# Maintain the native pip workflow for generated requirements-based starters.
if primary and (bundled/'wheelhouse').is_dir():
 shutil.rmtree(bundled/'wheelhouse');shutil.copytree(cache/'wheelhouse',bundled/'wheelhouse')
