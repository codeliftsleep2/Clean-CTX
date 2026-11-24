"""Verify E3 pins and emit only this increment's review patch; never mutate source."""
import difflib
import hashlib
import json
import re
import subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1/e3-complete-20261123"
hash_bytes=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((ROOT/"target/ops-r1/e3-build-r4-20261123/build-sources.json").read_text(encoding="utf-8"))
assert all(hash_bytes((ROOT/r["path"]).read_bytes())==r["sha256"] for r in manifest)
authority=json.loads((ROOT/"target/ops-r1/catalog-repair-20261123/acquire-requests.json").read_text(encoding="utf-8"))
assert all(hash_bytes((ROOT/r["path"]).read_bytes())==r["sourceSha256"] for q in authority for r in q["references"])
name="src/ir/typescript_resolution/source_observations.rs"
after=(ROOT/name).read_bytes()
before=re.sub(rb'#\[path = "request_budget.rs"\]\r?\npub mod request_budget;\r?\n',b'',after)
before=re.sub(rb'    source_bytes: usize,\r?\n',b'',before)
before=re.sub(rb'            source_bytes: bytes.len\(\),\r?\n',b'',before)
old=json.loads((ROOT/"target/ops-r1/e2-green-20261123/build-sources.json").read_text(encoding="utf-8"))
expected=next(r["sha256"] for r in old if r["path"]==name)
assert hash_bytes(before)==expected,"E2 predecessor reconstruction must match the archived exact hash"
patch=list(difflib.unified_diff(before.decode("utf-8").splitlines(True),after.decode("utf-8").splitlines(True),fromfile="a/"+name,tofile="b/"+name))
files=["src/ir/typescript_resolution/request_budget.rs","src/tests/ir/written_source_budget_e3.rs"]
files += [str(p.relative_to(ROOT)).replace('\\','/') for p in sorted((ROOT/"verification/ops-r1/e3").glob("*")) if p.is_file()]
checks=[]
for n in [name]+files:
    raw=(ROOT/n).read_bytes();text=raw.decode("utf-8",errors="strict")
    assert not raw.startswith(b'\xef\xbb\xbf')
    assert n.endswith('.md') or len(text.splitlines())<=615
    checks.append({"path":n,"sha256":hash_bytes(raw),"lines":len(text.splitlines())})
    if n!=name:patch.extend(difflib.unified_diff([],text.splitlines(True),fromfile="/dev/null",tofile="b/"+n))
(OUT/"implementation.diff").write_text(''.join(patch),encoding="utf-8",newline='')
assert hash_bytes((ROOT/"src/tests/ir/written_source_observations_e2.rs").read_bytes())=="8a4bd2bf47057c554e94e141e3865b387150e80b7f940f2b341a7b83dcbde3a8"
result=subprocess.run(["git","diff","--check"],cwd=ROOT,capture_output=True)
assert result.returncode==0
(OUT/"final-checks.json").write_text(json.dumps({"source_pins_current":True,"authority_pins_current":True,
    "e2_predecessor_hash":expected,"e2_regression_unchanged":True,"diff_check_passed":True,"files":checks},indent=2),encoding="utf-8")
print("Source/authority pins, exact E2 predecessor, encoding, file sizes, frozen E2 regression and diff checks passed")
