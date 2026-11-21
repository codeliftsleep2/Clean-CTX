"""Prepare one test-only source identity; never compile or execute tests."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
ISO = ROOT / 'target/ops-r1/m3-isolated-23'
OUT = Path(__file__).resolve().parent
EXEC = ISO / 'verification/ops-r1/proposals/shared-acquisition/execution'
VERSION = 'performance-25-cost-red'
BASE = json.loads((EXEC / 'reviewed-green-performance-25-baselineb-sources.json').read_bytes())
sha = lambda data: hashlib.sha256(data).hexdigest()


def rows(manifest):
    tracked = subprocess.check_output(
        ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
        cwd=ISO).decode().split('\0')
    result = []
    for name in sorted(set(filter(None, tracked)) | {p['path'] for p in manifest['files']}):
        data = (ISO / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        result.append({'path': name, 'sha256': sha(data)})
    return result


assert sha(json.dumps(rows(BASE), separators=(',', ':')).encode()) == BASE['candidate_source_tree_sha256']
history = OUT / 'measurement-baseline-b-inputs.zip'
assert not history.exists(), 'Refuse to overwrite historical source capture'
with zipfile.ZipFile(history, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
    for name in [
        'src/tests/mcp/ops_r1_absent_route.rs',
        'src/tests/architecture/ops_r1_observation.rs',
        'src/ir/typescript_resolution/host.rs',
        'src/workspace/index/coverage.rs',
        'src/tests/ir/ops_r1_shared_acquisition_controls.rs',
    ]:
        archive.write(ISO / name, name)
    for name in ['capture.py', 'reviewed-green-performance-25-baselineb-sources.json']:
        archive.write(EXEC / name, 'execution/' + name)
build = ISO / 'target/ops-r1/performance-25-baselineb-build-01/build-receipt.json'
receipt = json.loads(build.read_bytes())
binary = Path(receipt['executable'])
assert sha(binary.read_bytes()) == receipt['executable_sha256']
preserved = ROOT / 'target/ops-r1/performance-25-preserved-baselineb'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build, preserved / 'build-receipt.json')
for name in ['ops_r1_shared_acquisition_controls.rs', 'ops_r1_shared_acquisition_cost.rs']:
    relative = Path('src/tests/ir') / name
    shutil.copy2(ROOT / relative, ISO / relative)
manifest = dict(BASE)
manifest['standing'] = 'Test-only historical RED for exact imported constraint duplication; production unchanged'
manifest['files'] = BASE['files'] + [{'path': 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'}]
manifest['test_identities'] = BASE['test_identities'] + [
    'ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_import_compacts_exact_constraints_without_losing_receipts']
source_rows = rows(manifest)
manifest['candidate_source_tree_sha256'] = sha(json.dumps(source_rows, separators=(',', ':')).encode())
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
pin.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
adapter = EXEC / 'capture.py'
text = adapter.read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'performance-25-baselineb'):", "'performance-25-baselineb','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases={'cost-constraints':(1," + str(len(manifest['test_identities']) - 1) + ")}\n    core.CASES=", 1)
adapter.write_text(text, encoding='utf-8')
for path in [pin, adapter]:
    shutil.copy2(path, OUT / ('cost-red-' + path.name))
record = {
    'baseline_source_tree_sha256': BASE['candidate_source_tree_sha256'],
    'candidate_source_tree_sha256': manifest['candidate_source_tree_sha256'],
    'source_identity_algorithm': 'Sorted path and normalized source SHA256 rows, compact JSON SHA256; existing reviewed monitor',
    'manifest_sha256': sha(pin.read_bytes()),
    'adapter_sha256': sha(adapter.read_bytes()),
    'baseline_source_archive_sha256': sha(history.read_bytes()),
    'preserved_binary_sha256': sha((preserved / binary.name).read_bytes()),
    'production_changed': False,
    'tests_executed': False,
}
(OUT / 'cost-red-preparation.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(manifest['candidate_source_tree_sha256'])
