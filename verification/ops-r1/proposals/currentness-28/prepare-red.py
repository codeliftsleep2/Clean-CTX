"""Pin one retained-currentness regression against the validated per-call producer."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
BASE = ROOT / 'verification/ops-r1/proposals/validation-27/execution'
VERSION = 'currentness-28-red'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((BASE / 'reviewed-green-validation-27-green-sources.json').read_bytes())
test = 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'
with zipfile.ZipFile(ROOT / 'verification/ops-r1/proposals/validation-27/validation-and-measurement-evidence.zip') as prior:
    original = prior.read('root-source/' + test)
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})


def identity(baseline):
    rows = []
    for name in names:
        data = original if baseline and name == test else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


baseline = identity(True)
assert baseline == manifest['candidate_source_tree_sha256']
build_path = ROOT / 'target/ops-r1/validation-27-green-build-01/build-receipt.json'
build = json.loads(build_path.read_bytes())
binary = Path(build['executable'])
assert build['source_tree_sha256'] == baseline and sha(binary.read_bytes()) == build['executable_sha256']
preserved = ROOT / 'target/ops-r1/currentness-28-preserved-baseline'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build_path, preserved / 'build-receipt.json')
offset = len(manifest['test_identities'])
manifest['standing'] = 'Test-only RED for duplicate exact constraints in retained currentness; producer correction retained'
manifest['baseline_source_tree_sha256'] = baseline
manifest['candidate_source_tree_sha256'] = identity(False)
manifest['test_identities'] += ['ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_retained_currentness_checks_exact_constraints_once_per_call']
EXEC.mkdir(exist_ok=True)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (BASE / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'validation-27-green'):", "'validation-27-green','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases={'retained-currentness':(1," + str(offset) + ")}\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
(EXEC / 'invoke-build-capture.ps1').write_bytes((BASE / 'invoke-build-capture.ps1').read_bytes())
record = {'baseline_source_sha256': baseline, 'red_source_sha256': manifest['candidate_source_tree_sha256'],
          'test_file_sha256': sha((ROOT / test).read_bytes()), 'production_unchanged_from_validation_27': True,
          'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes())}
(OUT / 'red-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
