"""Preserve verified import candidate and prepare the separate directory-work RED."""
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
VERSION = 'performance-25-directory-red'
sha = lambda data: hashlib.sha256(data).hexdigest()
base = json.loads((EXEC / 'reviewed-green-performance-25-cost-green-sources.json').read_bytes())
tracked = subprocess.check_output(
    ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
    cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {p['path'] for p in base['files']})


def identity():
    rows = []
    for name in names:
        data = (ISO / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity() == base['candidate_source_tree_sha256']
build = ISO / 'target/ops-r1/performance-25-cost-green-build-01/build-receipt.json'
receipt = json.loads(build.read_bytes())
binary = Path(receipt['executable'])
assert sha(binary.read_bytes()) == receipt['executable_sha256']
preserved = ROOT / 'target/ops-r1/performance-25-preserved-cost-green'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build, preserved / 'build-receipt.json')
history = OUT / 'cost-verified-green.zip'
bundle = ROOT / 'target/ops-r1'
bundles = sorted(bundle.glob('cost-validation-*'))
assert bundles, 'Missing operator control receipts'
controls = json.loads((bundles[-1] / 'receipts.json').read_bytes())
assert len(controls) == 8
with zipfile.ZipFile(history, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.write(ISO / name, 'source/' + name)
    for entry in controls:
        path = Path(entry['receipt'])
        assert sha(path.read_bytes()) == entry['sha256']
        observed = json.loads(path.read_bytes())
        assert observed['status'] == 'TRACKED_TEST_COMPLETED'
        assert observed['source_tree_sha256'] == base['candidate_source_tree_sha256']
        for evidence in path.parent.iterdir():
            if evidence.is_file():
                archive.write(evidence, 'controls/' + entry['case'] + '/' + evidence.name)
    for path in [EXEC / 'capture.py', EXEC / 'reviewed-green-performance-25-cost-green-sources.json', build]:
        archive.write(path, 'pins/' + path.name)
relative = Path('src/tests/ir/ops_r1_shared_acquisition_cost.rs')
shutil.copy2(ROOT / relative, ISO / relative)
manifest = dict(base)
manifest['standing'] = 'Separate directory-work RED; verified exact-import correction retained'
manifest['candidate_source_tree_sha256'] = identity()
manifest['test_identities'] = base['test_identities'] + [
    'ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_empty_automatic_types_reuses_project_across_owner_directories']
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
pin.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
adapter = EXEC / 'capture.py'
text = adapter.read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'performance-25-cost-green'):", "'performance-25-cost-green','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases={'directory-work':(1,39)}\n    core.CASES=", 1)
adapter.write_text(text, encoding='utf-8')
for path in [pin, adapter]:
    shutil.copy2(path, OUT / ('directory-red-' + path.name))
record = {
    'baseline_source_sha256': base['candidate_source_tree_sha256'],
    'candidate_source_tree_sha256': manifest['candidate_source_tree_sha256'],
    'manifest_sha256': sha(pin.read_bytes()),
    'adapter_sha256': sha(adapter.read_bytes()),
    'verified_import_green_archive_sha256': sha(history.read_bytes()),
    'preserved_import_green_binary_sha256': receipt['executable_sha256'],
    'production_changed_from_import_green': False,
    'new_test_identity': manifest['test_identities'][-1],
}
(OUT / 'directory-red-preparation.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(manifest['candidate_source_tree_sha256'])
