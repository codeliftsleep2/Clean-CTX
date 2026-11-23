"""Apply only the validated producer change to the existing measurement workspace."""
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
ISO = ROOT / 'target/ops-r1/m3-isolated-23'
EXEC = ISO / 'verification/ops-r1/proposals/shared-acquisition/execution'
VERSION = 'validation-27-performance'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((EXEC / 'reviewed-green-performance-25-directory-green-sources.json').read_bytes())
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})


def identity():
    rows = []
    for name in names:
        data = (ISO / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


baseline = identity()
assert baseline == manifest['candidate_source_tree_sha256']
prior_path = ISO / 'target/ops-r1/performance-25-directory-green-build-01/build-receipt.json'
prior = json.loads(prior_path.read_bytes())
binary = Path(prior['executable'])
assert sha(binary.read_bytes()) == prior['executable_sha256']
preserved = ROOT / 'target/ops-r1/validation-27-preserved-measurement-baseline'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(prior_path, preserved / 'build-receipt.json')
for name in ['capture.py', 'reviewed-green-performance-25-directory-green-sources.json']:
    shutil.copy2(EXEC / name, OUT / ('measurement-baseline-' + name))
changed = 'src/ir/typescript_resolution/shared_acquisition.rs'
(OUT / 'measurement-baseline-shared_acquisition.rs').write_bytes((ISO / changed).read_bytes())
root_build = json.loads((ROOT / 'target/ops-r1/validation-27-green-build-01/build-receipt.json').read_bytes())
root_pin = json.loads((OUT / 'execution/reviewed-green-validation-27-green-sources.json').read_bytes())
assert root_build['source_tree_sha256'] == root_pin['candidate_source_tree_sha256']
root_names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
root_rows = []
for name in sorted(set(filter(None, root_names)) | {entry['path'] for entry in root_pin['files']}):
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    root_rows.append({'path': name, 'sha256': sha(data)})
assert sha(json.dumps(root_rows, separators=(',', ':')).encode()) == root_build['source_tree_sha256']
assert sha((ROOT / changed).read_bytes()) != sha((ISO / changed).read_bytes())
shutil.copy2(ROOT / changed, ISO / changed)
manifest['standing'] = 'Same measurement fixture/instrumentation; only validated per-call producer correction differs'
manifest['baseline_source_tree_sha256'] = baseline
manifest['candidate_source_tree_sha256'] = identity()
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'performance-25-directory-green'):", "'performance-25-directory-green','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases=CASES_PERFORMANCE_25\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
for path in [pin, EXEC / 'capture.py']:
    shutil.copy2(path, OUT / ('measurement-candidate-' + path.name))
record = {'baseline_measurement_source_sha256': baseline,
          'candidate_measurement_source_sha256': manifest['candidate_source_tree_sha256'],
          'validated_root_source_sha256': root_build['source_tree_sha256'],
          'producer_sha256': sha((ROOT / changed).read_bytes()),
          'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
          'changed_production_file': changed, 'fixture_and_instrumentation_unchanged': True}
(OUT / 'measurement-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
