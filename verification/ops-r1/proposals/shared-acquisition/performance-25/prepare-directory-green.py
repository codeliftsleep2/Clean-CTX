"""Pin the directory correction while retaining the exact RED and executable."""
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
VERSION = 'performance-25-directory-green'
sha = lambda data: hashlib.sha256(data).hexdigest()
red = json.loads((EXEC / 'reviewed-green-performance-25-directory-red-sources.json').read_bytes())
tracked = subprocess.check_output(
    ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
    cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {p['path'] for p in red['files']})
changed = 'src/ir/typescript_resolution/shared_acquisition.rs'
candidate = (ISO / changed).read_bytes().replace(b'\r\n', b'\n')
begin = candidate.index(b'    // Automatic type discovery starts')
end = candidate.index(b'    serde_json::to_string', begin)
original = candidate[:begin] + candidate[end:]
original = original.replace(
    b'/// Preserve directory-dependent lookup unless explicit types disable that lookup.',
    b'/// Conservative basis: preserve logical routes; different request directories split.')
original = original.replace(b'"directory": directory,', b'"directory": file.parent(),')


def identity(before):
    rows = []
    for name in names:
        data = original if before and name == changed else (ISO / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity(True) == red['candidate_source_tree_sha256']
build = ISO / 'target/ops-r1/performance-25-directory-red-build-01/build-receipt.json'
receipt = json.loads(build.read_bytes())
binary = Path(receipt['executable'])
assert sha(binary.read_bytes()) == receipt['executable_sha256']
preserved = ROOT / 'target/ops-r1/performance-25-preserved-directory-red'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build, preserved / 'build-receipt.json')
failure = ISO / 'target/ops-r1/p0-directory-work-20261121-011248'
observed = json.loads((failure / 'receipt.json').read_bytes())
assert observed['exit_code'] == 101 and observed['post_source_and_binary_identity_verified']
assert observed['executable_sha256'] == receipt['executable_sha256']
assert 'left: [2, 2, 4]' in (failure / 'stderr.txt').read_text(encoding='utf-8')
history = OUT / 'directory-historical-red.zip'
with zipfile.ZipFile(history, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.writestr('source/' + name, original if name == changed else (ISO / name).read_bytes())
    for evidence in failure.iterdir():
        if evidence.is_file():
            archive.write(evidence, 'execution/' + evidence.name)
    for path in [EXEC / 'capture.py', EXEC / 'reviewed-green-performance-25-directory-red-sources.json', build]:
        archive.write(path, 'pins/' + path.name)
manifest = dict(red)
manifest['standing'] = 'Explicit empty automatic types permit directory-independent qualification; no freshness bypass'
manifest['candidate_source_tree_sha256'] = identity(False)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
pin.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
adapter = EXEC / 'capture.py'
text = adapter.read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'performance-25-directory-red'):", "'performance-25-directory-red','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases=dict(CASES_PERFORMANCE_25, **{'cost-constraints':(1,38),'directory-work':(1,39)})\n    core.CASES=", 1)
adapter.write_text(text, encoding='utf-8')
for path in [pin, adapter]:
    shutil.copy2(path, OUT / ('directory-green-' + path.name))
record = {
    'historical_red_source_sha256': red['candidate_source_tree_sha256'],
    'candidate_source_tree_sha256': manifest['candidate_source_tree_sha256'],
    'manifest_sha256': sha(pin.read_bytes()),
    'adapter_sha256': sha(adapter.read_bytes()),
    'historical_red_archive_sha256': sha(history.read_bytes()),
    'preserved_red_binary_sha256': receipt['executable_sha256'],
    'changed_production_file': changed,
    'test_sha256': sha((ISO / 'src/tests/ir/ops_r1_shared_acquisition_cost.rs').read_bytes()),
    'tests_executed': False,
}
(OUT / 'directory-green-preparation.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(manifest['candidate_source_tree_sha256'])
