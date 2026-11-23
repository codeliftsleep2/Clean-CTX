"""Pin test-only RED for repeated constraints inside one validation boundary."""
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
BASE = ROOT / 'verification/ops-r1/proposals/integration-26/execution'
VERSION = 'validation-27-red'
sha = lambda value: hashlib.sha256(value).hexdigest()
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
assert head == 'dbf495517db50cc83fab747c34b33ff989704e27'
manifest = json.loads((BASE / 'reviewed-green-integration-26-sources.json').read_bytes())
test = 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'
changed = subprocess.check_output(['git', 'diff', '--name-only', 'HEAD', '--', 'src'], cwd=ROOT).decode().splitlines()
assert changed == [test], 'Only the new tracked work assertions may differ in this RED'
tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {entry['path'] for entry in manifest['files']})
original = subprocess.check_output(['git', 'show', 'HEAD:' + test], cwd=ROOT)


def identity(baseline):
    rows = []
    for name in names:
        data = original if baseline and name == test else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity(True) == manifest['candidate_source_tree_sha256']
prior_build = ROOT / 'target/ops-r1/integration-26-build-01/build-receipt.json'
prior = json.loads(prior_build.read_bytes())
binary = Path(prior['executable'])
assert sha(binary.read_bytes()) == prior['executable_sha256']
preserved = ROOT / 'target/ops-r1/validation-27-preserved-baseline'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(prior_build, preserved / 'build-receipt.json')
manifest['standing'] = 'Test-only RED: exact constraint checks within each fresh sealing/qualification call'
manifest['baseline_source_tree_sha256'] = identity(True)
manifest['candidate_source_tree_sha256'] = identity(False)
offset = len(manifest['test_identities'])
manifest['test_identities'] += [
    'ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_seal_checks_each_exact_constraint_once_per_call',
    'ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_qualification_checks_each_exact_constraint_once_per_call',
]
EXEC.mkdir(exist_ok=True)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists(), 'Never overwrite a source pin'
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (BASE / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'integration-26'):", "'integration-26','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("head!='a59d361e415861327412c8a25de5807003c8639e'", "head!='" + head + "'")
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases={'seal':(1," + str(offset) + "),'qualification':(1," + str(offset + 1) + ")}\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
(EXEC / 'invoke-build-capture.ps1').write_bytes((BASE / 'invoke-build-capture.ps1').read_bytes())
record = {
    'baseline_source_sha256': manifest['baseline_source_tree_sha256'],
    'red_source_sha256': manifest['candidate_source_tree_sha256'],
    'source_pin_sha256': sha(pin.read_bytes()),
    'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
    'new_test_file_sha256': sha((ROOT / test).read_bytes()),
    'production_unchanged': True, 'preserved_baseline_binary_sha256': prior['executable_sha256'],
}
(OUT / 'red-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
