"""Preserve the exact historical RED and pin the scoped import correction."""
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
VERSION = 'performance-25-cost-green'
sha = lambda data: hashlib.sha256(data).hexdigest()
red = json.loads((EXEC / 'reviewed-green-performance-25-cost-red-sources.json').read_bytes())
tracked = subprocess.check_output(
    ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
    cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {p['path'] for p in red['files']})
host = 'src/ir/typescript_resolution/host.rs'
inputs = 'src/ir/typescript_resolution/acquisition_inputs.rs'
originals = {}
originals[host] = (ISO / host).read_bytes().replace(
    b'Eq, PartialEq, Hash, serde::Serialize', b'Eq, PartialEq, serde::Serialize')
new = '''        // Identical constraints have the same currentness predicate. Keep the
        // first occurrence without collapsing distinct identities or receipts.
        let mut known: std::collections::HashSet<_> = self.values.iter().cloned().collect();
        for witness in witnesses {
            if known.insert(witness.clone()) {
                self.values.push(witness);
            }
        }
'''.encode()
current = (ISO / inputs).read_bytes().replace(b'\r\n', b'\n')
assert current.count(new) == 1
originals[inputs] = current.replace(new, b'        self.values.extend(witnesses);\n')


def identity(overrides):
    rows = []
    for name in names:
        data = overrides[name] if name in overrides else (ISO / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity(originals) == red['candidate_source_tree_sha256'], 'Historical RED reconstruction mismatch'
build = ISO / 'target/ops-r1/performance-25-cost-red-build-01/build-receipt.json'
receipt = json.loads(build.read_bytes())
binary = Path(receipt['executable'])
assert sha(binary.read_bytes()) == receipt['executable_sha256']
preserved = ROOT / 'target/ops-r1/performance-25-preserved-cost-red'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build, preserved / 'build-receipt.json')
failure = ISO / 'target/ops-r1/p0-cost-constraints-20261121-002440'
observed = json.loads((failure / 'receipt.json').read_bytes())
assert observed['source_tree_sha256'] == red['candidate_source_tree_sha256']
assert observed['executable_sha256'] == receipt['executable_sha256']
assert observed['exit_code'] == 101 and observed['post_source_and_binary_identity_verified']
assert 'left: 3' in (failure / 'stderr.txt').read_text(encoding='utf-8')
history = OUT / 'cost-historical-red.zip'
with zipfile.ZipFile(history, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.writestr('source/' + name, originals[name] if name in originals else (ISO / name).read_bytes())
    for path in failure.iterdir():
        if path.is_file():
            archive.write(path, 'execution/' + path.name)
    for path in [EXEC / 'capture.py', EXEC / 'reviewed-green-performance-25-cost-red-sources.json', build]:
        archive.write(path, 'pins/' + path.name)
manifest = dict(red)
manifest['standing'] = 'Scoped exact shared-import constraint compaction; unchanged RED regression'
manifest['candidate_source_tree_sha256'] = identity({})
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
pin.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
adapter = EXEC / 'capture.py'
text = adapter.read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'performance-25-cost-red'):", "'performance-25-cost-red','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases=dict(CASES_PERFORMANCE_25, **{'cost-constraints':(1,38)})\n    core.CASES=", 1)
adapter.write_text(text, encoding='utf-8')
for path in [pin, adapter]:
    shutil.copy2(path, OUT / ('cost-green-' + path.name))
record = {
    'historical_red_source_sha256': red['candidate_source_tree_sha256'],
    'candidate_source_tree_sha256': manifest['candidate_source_tree_sha256'],
    'manifest_sha256': sha(pin.read_bytes()),
    'adapter_sha256': sha(adapter.read_bytes()),
    'historical_red_archive_sha256': sha(history.read_bytes()),
    'preserved_red_binary_sha256': receipt['executable_sha256'],
    'test_sha256': sha((ISO / 'src/tests/ir/ops_r1_shared_acquisition_cost.rs').read_bytes()),
    'production_changed_files': [host, inputs],
    'tests_executed': False,
}
(OUT / 'cost-green-preparation.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(manifest['candidate_source_tree_sha256'])
