"""Pin the test-only repeated-request extension without changing producer behavior."""
import hashlib,json,subprocess,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
OUT=Path(__file__).resolve().parent
EXEC=OUT/'execution'
VERSION='retention-30'
sha=lambda b:hashlib.sha256(b).hexdigest()
prior=ROOT/'verification/ops-r1/proposals/phase-29/execution/reviewed-green-phase-29-green-sources.json'
manifest=json.loads(prior.read_bytes())
changed='src/tests/mcp/ops_r1_acquisition_fitness.rs'
tracked=subprocess.check_output(['git','ls-files','-z','--','src','Cargo.toml','Cargo.lock','build.rs','.cargo'],cwd=ROOT).decode().split('\0')
names=sorted(set(filter(None,tracked))|{f['path'] for f in manifest['files']})
rows=[]
baseline_rows=[]
with zipfile.ZipFile(ROOT/'verification/ops-r1/proposals/phase-29/validation-and-measurement-evidence.zip') as history:
 for name in names:
  data=(ROOT/name).read_bytes()
  if name.startswith('src/') and name.endswith('.rs'):
   old=history.read('root-source/'+name)
   if name!=changed:assert data.replace(b'\r\n',b'\n')==old.replace(b'\r\n',b'\n'),name
  if Path(name).suffix in ('.rs','.toml') or name=='Cargo.lock':data=data.replace(b'\r\n',b'\n')
  rows.append({'path':name,'sha256':sha(data)})
  previous=history.read('root-source/'+name).replace(b'\r\n',b'\n') if name==changed else data
  baseline_rows.append({'path':name,'sha256':sha(previous)})
assert sha(json.dumps(baseline_rows,separators=(',',':')).encode())==manifest['candidate_source_tree_sha256']
offset=len(manifest['test_identities'])
manifest['test_identities'].append('mcp::semantic_compilation::ops_r1_acquisition_fitness::ops_r1_repeated_absent_queries_preserve_retained_owner_evidence')
manifest['baseline_source_tree_sha256']=manifest['candidate_source_tree_sha256']
manifest['candidate_source_tree_sha256']=sha(json.dumps(rows,separators=(',',':')).encode())
manifest['standing']='Test-only repeated distinct absent selectors, one state; no production changes'
pin=EXEC/('reviewed-green-'+VERSION+'-sources.json');assert not pin.exists()
pin.write_bytes((json.dumps(manifest,indent=2)+'\n').encode())
p=EXEC/'capture.py';text=p.read_text(encoding='utf-8')
text=text.replace('SOURCE_PINS={',"SOURCE_PINS={'green-"+VERSION+"': '"+sha(pin.read_bytes())+"', ",1)
text=text.replace('if amendment not in (',"if amendment not in ('"+VERSION+"',",1)
text=text.replace('    proposal=pathlib',"        if amendment=='"+VERSION+"':source_key='green-"+VERSION+"'\n    proposal=pathlib",1)
text=text.replace('    core.CASES=',"    if source_key=='green-"+VERSION+"':selected_cases={'repeated-requests':(4,"+str(offset)+")}\n    core.CASES=",1)
compile(text,str(p),'exec');p.write_bytes(text.encode())
record={'source_tree_sha256':manifest['candidate_source_tree_sha256'],'baseline_source_tree_sha256':manifest['baseline_source_tree_sha256'],'changed_source_files':[changed],'producer_unchanged':True,'adapter_sha256':sha(p.read_bytes()),'source_pin_sha256':sha(pin.read_bytes())}
(OUT/'preparation.json').write_bytes((json.dumps(record,indent=2)+'\n').encode())
print(json.dumps(record,indent=2))
