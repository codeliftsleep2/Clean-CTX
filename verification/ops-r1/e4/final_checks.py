"""Verify exact E3 predecessors and emit an E4-only implementation patch."""
import difflib
import hashlib
import json
import re
import subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1/e4-complete-20261123"
digest=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((ROOT/"target/ops-r1/e4-build-r5-20261123/build-sources.json").read_text(encoding="utf-8"))
assert all(digest((ROOT/r["path"]).read_bytes())==r["sha256"] for r in manifest)
old={r["path"]:r["sha256"] for r in json.loads((ROOT/"target/ops-r1/e3-build-r4-20261123/build-sources.json").read_text(encoding="utf-8"))}
base="src/ir/typescript_resolution/"
changes={}
n=base+"source_observations.rs";after=(ROOT/n).read_bytes()
before=re.sub(rb'#\[path = "direct_identity.rs"\]\r?\npub mod direct_identity;\r?\n',b'',after)
changes[n]=(before,after)
n=base+"mod.rs";after=(ROOT/n).read_bytes()
before=re.sub(rb'#\[cfg\(feature = "written_source_observations"\)\]\r?\nmod host_direct_budget;\r?\n',b'',after)
changes[n]=(before,after)
n=base+"request_budget.rs";after=(ROOT/n).read_bytes()
lines=(ROOT/"target/ops-r1/e3-complete-20261123/implementation.diff").read_bytes().decode("utf-8").splitlines(True)
collect=False;before=[]
for line in lines:
    if line.startswith('+++ '):
        collect=line.strip()=='+++ b/'+n
    elif collect and line.startswith('--- '):break
    elif collect and line.startswith('+'):before.append(line[1:])
changes[n]=(''.join(before).encode("utf-8"),after)
n=base+"host.rs";after=(ROOT/n).read_bytes();before=after
before=re.sub(rb'    #\[cfg\(feature = "written_source_observations"\)\]\r?\n    pub\(super\) direct_[^\r\n]+\r?\n',b'',before)
before=re.sub(rb'            #\[cfg\(feature = "written_source_observations"\)\]\r?\n            direct_[^\r\n]+\r?\n',b'',before)
before=re.sub(rb'        #\[cfg\(feature = "written_source_observations"\)\]\r?\n        let entries = if self.direct_budget.is_some\(\) \{.*?        #\[cfg\(not\(feature = "written_source_observations"\)\)\]\r?\n',b'',before,flags=re.S)
before=re.sub(rb'        #\[cfg\(feature = "written_source_observations"\)\]\r?\n        if self.direct_budget.is_some\(\) \{\r?\n            return self.read_direct_bytes\(path, file\);\r?\n        \}\r?\n',b'',before)
changes[n]=(before,after)
patch=[];checks=[]
for n,(before,after) in changes.items():
    assert digest(before)==old[n],f"E3 predecessor reconstruction differs: {n}"
    patch.extend(difflib.unified_diff(before.decode("utf-8").splitlines(True),after.decode("utf-8").splitlines(True),fromfile='a/'+n,tofile='b/'+n))
new=[base+"direct_identity.rs",base+"host_direct_budget.rs","src/tests/ir/written_source_identity_e4.rs"]
new.extend(str(p.relative_to(ROOT)).replace('\\','/') for p in sorted((ROOT/"verification/ops-r1/e4").glob('*')) if p.is_file())
for n in new:
    text=(ROOT/n).read_bytes().decode("utf-8")
    patch.extend(difflib.unified_diff([],text.splitlines(True),fromfile='/dev/null',tofile='b/'+n))
for n in list(changes)+new:
    b=(ROOT/n).read_bytes();s=b.decode("utf-8",errors="strict")
    assert not b.startswith(b'\xef\xbb\xbf')
    assert n.endswith('.md') or len(s.splitlines())<=615
    checks.append({"path":n,"sha256":digest(b),"lines":len(s.splitlines())})
authority=json.loads((ROOT/"target/ops-r1/catalog-repair-20261123/acquire-requests.json").read_text(encoding="utf-8"))
assert all(digest((ROOT/r["path"]).read_bytes())==r["sourceSha256"] for q in authority for r in q["references"])
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
(OUT/"implementation.diff").write_text(''.join(patch),encoding="utf-8",newline='')
(OUT/"final-checks.json").write_text(json.dumps({"source_pins_current":True,"authority_pins_current":True,"exact_e3_predecessors_verified":True,"files":checks},indent=2),encoding="utf-8")
print('E4 source/authority pins, exact E3 predecessors, encoding, active sizes and diff checks passed')
