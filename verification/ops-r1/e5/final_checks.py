"""Verify E5 source pins/authority and reconstruct exact E4 rollback boundaries."""
import difflib
import hashlib
import json
import re
import subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1/e5-final-20261123"
digest=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((ROOT/"target/ops-r1/e5-build-r7-20261123/build-sources.json").read_text(encoding="utf-8"))
assert all(digest((ROOT/r['path']).read_bytes())==r['sha256'] for r in manifest)
old={r['path']:r['sha256'] for r in json.loads((ROOT/"target/ops-r1/e4-build-r5-20261123/build-sources.json").read_text(encoding="utf-8"))}
base='src/ir/typescript_resolution/'
changes={}
n=base+'source_observations.rs';after=(ROOT/n).read_bytes()
before=re.sub(rb'#\[cfg\(feature = "angular"\)\]\r?\npub use crate::angular_meta::written_claim_composition as claim_composition;\r?\n',b'',after)
changes[n]=(before,after)
n=base+'mod.rs';after=(ROOT/n).read_bytes();changes[n]=(after.replace(b'pub(crate) mod e1_metrics;',b'pub(super) mod e1_metrics;'),after)
n='src/tests/ir/ops_r1_e1_metrics.rs';after=(ROOT/n).read_bytes();changes[n]=(after.replace(b'pub(crate)',b'pub(super)'),after)
n=base+'direct_identity.rs';after=(ROOT/n).read_bytes()
lines=(ROOT/'target/ops-r1/e4-complete-20261123/implementation.diff').read_bytes().decode('utf-8').splitlines(True)
collect=False;before=[]
for line in lines:
    if line.startswith('+++ '):collect=line.strip()=='+++ b/'+n
    elif collect and line.startswith('--- '):break
    elif collect and line.startswith('+'):before.append(line[1:])
changes[n]=(''.join(before).encode('utf-8'),after)
n='src/angular_meta/mod.rs';after=(ROOT/n).read_bytes()
stripped=re.sub(rb'#\[cfg\(feature = "written_source_observations"\)\]\r?\npub mod written_claim_composition;\r?\n',b'',after)
candidates=[b'// src/angular_meta/mod.rs'+eol+b'//'+eol+stripped for eol in [b'\r\n',b'\n']]
before=next((b for b in candidates if digest(b)==old[n]),None)
assert before is not None,'Angular module predecessor does not match'
changes[n]=(before,after)
patch=[];checks=[]
for n,(before,after) in changes.items():
    assert digest(before)==old[n],n
    patch.extend(difflib.unified_diff(before.decode('utf-8').splitlines(True),after.decode('utf-8').splitlines(True),fromfile='a/'+n,tofile='b/'+n))
new=['src/angular_meta/written_claim_composition.rs',base+'role_annotation.rs','src/tests/ir/written_claim_composition_e5.rs']
new.extend(str(p.relative_to(ROOT)).replace('\\','/') for p in sorted((ROOT/'verification/ops-r1/e5').glob('*')) if p.is_file())
for n in new:patch.extend(difflib.unified_diff([],(ROOT/n).read_bytes().decode('utf-8').splitlines(True),fromfile='/dev/null',tofile='b/'+n))
for n in list(changes)+new:
    b=(ROOT/n).read_bytes();s=b.decode('utf-8',errors='strict');assert not b.startswith(b'\xef\xbb\xbf')
    assert n.endswith('.md') or len(s.splitlines())<=615,(n,len(s.splitlines()))
    checks.append({'path':n,'sha256':digest(b),'lines':len(s.splitlines())})
authority=json.loads((ROOT/'target/ops-r1/catalog-repair-20261123/acquire-requests.json').read_text(encoding='utf-8'))
assert all(digest((ROOT/r['path']).read_bytes())==r['sourceSha256'] for q in authority for r in q['references'])
assert digest((ROOT/'src/tests/ir/written_source_observations_e2.rs').read_bytes())=='8a4bd2bf47057c554e94e141e3865b387150e80b7f940f2b341a7b83dcbde3a8'
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
(OUT/'implementation.diff').write_text(''.join(patch),encoding='utf-8',newline='')
(OUT/'final-checks.json').write_text(json.dumps({'source_pins_current':True,'authority_pins_current':True,'e4_exact_predecessors_verified':True,
    'frozen_e2_regression_unchanged':True,'files':checks},indent=2),encoding='utf-8')
print('Exact E4 predecessors, E5 source/authority pins, frozen E2 regression, UTF-8, sizes and diff checks passed')
