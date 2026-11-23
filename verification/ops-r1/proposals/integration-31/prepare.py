"""Reconcile source, scope and exact reusable verification for Directive 31."""
import csv,hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
OUT=Path(sys.argv[1]).resolve()
HERE=Path(__file__).resolve().parent
ACCEPTED='679a676fe2d68641ee2b52e868e15a89c8b706052248b507f169c424573a6b3d'
sha=lambda b:hashlib.sha256(b).hexdigest()
prior=ROOT/'verification/ops-r1/proposals/retention-30/execution/reviewed-green-retention-30-sources.json'
m=json.loads(prior.read_bytes())
tracked=subprocess.check_output(['git','ls-files','-z','--','src','Cargo.toml','Cargo.lock','build.rs','.cargo'],cwd=ROOT).decode().split('\0')
rows=[]
for name in sorted(set(filter(None,tracked))|{r['path'] for r in m['files']}):
 b=(ROOT/name).read_bytes()
 if Path(name).suffix in ('.rs','.toml') or name=='Cargo.lock':b=b.replace(b'\r\n',b'\n')
 rows.append({'path':name,'sha256':sha(b)})
source=sha(json.dumps(rows,separators=(',',':')).encode())
files=set(subprocess.check_output(['git','diff','--name-only','HEAD','--'],cwd=ROOT).decode().splitlines())
files.update(subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=ROOT).decode().splitlines())
with (OUT/'scope.csv').open('w',newline='',encoding='utf-8') as f:
 w=csv.writer(f);w.writerow(['path','category','sha256'])
 for name in sorted(files):
  assert name.startswith(('src/','docs/agent/','verification/ops-r1/proposals/')),name
  p=ROOT/name;assert p.suffix!='.pyc',name
  category='regression' if name.startswith('src/tests/') else 'production' if name.startswith('src/') else 'governance/evidence'
  w.writerow([name,category,sha(p.read_bytes()) if p.exists() else 'DELETED'])
(OUT/'source-files.json').write_bytes((json.dumps(rows,indent=2)+'\n').encode())
(OUT/'review.diff').write_bytes(subprocess.check_output(['git','diff','HEAD','--','src','docs/agent'],cwd=ROOT))
record={'agent':'Agent MaxHeadRoom','source_tree_sha256':source,'source_changed':source!=ACCEPTED,'accepted_source_sha256':ACCEPTED,'scope_sha256':sha((OUT/'scope.csv').read_bytes())}
if source==ACCEPTED:
 build=ROOT/'target/ops-r1/retention-30-build-01/build-receipt.json';b=json.loads(build.read_bytes())
 assert b['source_tree_sha256']==source and b['exit_code']==0
 assert sha(Path(b['executable']).read_bytes())==b['executable_sha256']
 record.update(build_receipt=str(build),executable_sha256=b['executable_sha256'],verification_standing='Reuse exact retention-30 build and test; prior 18 regressions remain attributed to phase-29 producer-identical source')
else:
 target=HERE/'execution';pin=target/'reviewed-green-integration-31-sources.json';assert not pin.exists()
 m['candidate_source_tree_sha256']=source
 m['standing']='Integration formatting successor, no historical evidence relabeling'
 pin.write_bytes((json.dumps(m,indent=2)+'\n').encode())
 p=target/'capture.py';s=p.read_text(encoding='utf-8')
 s=s.replace('SOURCE_PINS={',"SOURCE_PINS={'green-integration-31': '"+sha(pin.read_bytes())+"', ",1)
 s=s.replace('if amendment not in (',"if amendment not in ('integration-31',",1)
 s=s.replace('    proposal=pathlib',"        if amendment=='integration-31':source_key='green-integration-31'\n    proposal=pathlib",1)
 cases="dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37),'seal':(1,38),'qualification':(1,39),'retained-currentness':(1,40),'public-work':(1,41),'phase-edit':(1,42),'conflicting-facts':(1,43),'repeated-requests':(4,44)})"
 s=s.replace('    core.CASES=',"    if source_key=='green-integration-31':selected_cases="+cases+"\n    core.CASES=",1)
 compile(s,str(p),'exec');p.write_bytes(s.encode())
record['execution']=str(HERE/'execution')
(OUT/'source.json').write_bytes((json.dumps(record,indent=2)+'\n').encode())
print(json.dumps(record,indent=2))
