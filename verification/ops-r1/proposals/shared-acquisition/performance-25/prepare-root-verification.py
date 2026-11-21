"""Pin the formatted root successor for the existing bounded monitor; no execution."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
OUT = Path(__file__).resolve().parent
EXEC = OUT.parent / 'execution'
VERSION = 'performance-25-root'
sha = lambda data: hashlib.sha256(data).hexdigest()
base = json.loads((EXEC / 'reviewed-green-hardening-24e-sources.json').read_bytes())
manifest = dict(base)
manifest['standing'] = 'Formatted root successor; independent focused verification, not relabeled isolated execution'
manifest['files'] = base['files'] + [{'path': 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'}]
manifest['test_identities'] = base['test_identities'] + [
    'ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_import_compacts_exact_constraints_without_losing_receipts',
    'ir::typescript_resolution::ops_r1_shared_acquisition_controls::cost::ops_r1_shared_empty_automatic_types_reuses_project_across_owner_directories',
]
assert len(base['test_identities']) == 36
paths = subprocess.check_output(
    ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
    cwd=ROOT).decode().split('\0')
rows = []
names = sorted(set(filter(None, paths)) | {p['path'] for p in manifest['files']})
for name in names:
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
manifest['candidate_source_tree_sha256'] = sha(json.dumps(rows, separators=(',', ':')).encode())
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists(), 'Do not overwrite a verification identity'
adapter = EXEC / 'capture.py'
original = adapter.read_bytes()
assert sha(original) == 'ae22421e64dcab69a48470bc44a92d3c433b27e2394476f3ec463399e8f1fbd1'
(OUT / 'root-pre-performance-adapter.py').write_bytes(original)
pin.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
text = original.decode('utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'hardening-24e'):", "'hardening-24e','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases=dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37)})\n    core.CASES=", 1)
adapter.write_text(text, encoding='utf-8')
for path in [pin, adapter]:
    shutil.copy2(path, OUT / ('root-' + path.name))
archive_path = OUT / 'formatted-root-source.zip'
with zipfile.ZipFile(archive_path, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.write(ROOT / name, 'source/' + name)
    archive.write(pin, 'pins/' + pin.name)
    archive.write(adapter, 'pins/capture.py')
record = {
    'accepted_baseline_sha256': 'c9a8c4eb39fdbde19a34e320cfcc5a8222adbc91940af8aa9ce9b09f24c1ab7f',
    'candidate_source_tree_sha256': manifest['candidate_source_tree_sha256'],
    'manifest_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha(adapter.read_bytes()),
    'source_archive_sha256': sha(archive_path.read_bytes()),
    'root_test_sha256': sha((ROOT / 'src/tests/ir/ops_r1_shared_acquisition_cost.rs').read_bytes()),
    'isolated_test_sha256': sha((ROOT / 'target/ops-r1/m3-isolated-23/src/tests/ir/ops_r1_shared_acquisition_cost.rs').read_bytes()),
    'standing': 'Root compilation and focused execution pending; historical evidence retains original identities',
}
(OUT / 'root-verification-preparation.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(manifest['candidate_source_tree_sha256'])
