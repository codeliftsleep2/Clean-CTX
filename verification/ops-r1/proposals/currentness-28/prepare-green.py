"""Preserve the exact retained-currentness RED and pin the scoped correction."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
VERSION = 'currentness-28-green'
sha = lambda data: hashlib.sha256(data).hexdigest()
red_path = EXEC / 'reviewed-green-currentness-28-red-sources.json'
red = json.loads(red_path.read_bytes())
red_record = json.loads((OUT / 'red-preparation.json').read_bytes())
test = 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'
assert sha((ROOT / test).read_bytes()) == red_record['test_file_sha256']
changed = ['src/ir/typescript_resolution/host.rs', 'src/workspace/index/coverage.rs',
           'src/workspace/index/resolver_registrations.rs', 'src/mcp/semantic_compilation/retained.rs']
with zipfile.ZipFile(ROOT / 'verification/ops-r1/proposals/validation-27/validation-and-measurement-evidence.zip') as prior:
    originals = {name: prior.read('root-source/' + name) for name in changed}
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in red['files']})


def identity(baseline):
    rows = []
    for name in names:
        data = originals[name] if baseline and name in originals else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity(True) == red['candidate_source_tree_sha256']
build_path = ROOT / 'target/ops-r1/currentness-28-red-build-01/build-receipt.json'
build = json.loads(build_path.read_bytes())
binary = Path(build['executable'])
assert sha(binary.read_bytes()) == build['executable_sha256']
preserved = ROOT / 'target/ops-r1/currentness-28-preserved-red'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build_path, preserved / 'build-receipt.json')
failure = ROOT / 'target/ops-r1/p0-retained-currentness-20261122-092627'
receipt = json.loads((failure / 'receipt.json').read_bytes())
assert receipt['exit_code'] == 101 and receipt['post_source_and_binary_identity_verified']
assert receipt['source_tree_sha256'] == red['candidate_source_tree_sha256']
assert 'left: 104' in (failure / 'stderr.txt').read_text(encoding='utf-8')
archive = OUT / 'historical-red.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.writestr('source/' + name, originals[name] if name in originals else (ROOT / name).read_bytes())
    for path in sorted(failure.rglob('*')):
        if path.is_file():
            saved.write(path, 'execution/' + path.relative_to(failure).as_posix())
    for path in [red_path, EXEC / 'capture.py', EXEC / 'invoke-build-capture.ps1', build_path]:
        saved.write(path, 'pins/' + path.name)
manifest = dict(red)
manifest['standing'] = 'Exact constraint checks within each currentness call; no result reuse across owners or boundaries'
manifest['candidate_source_tree_sha256'] = identity(False)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'currentness-28-red'):", "'currentness-28-red','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases=dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37),'seal':(1,38),'qualification':(1,39),'retained-currentness':(1,40)})\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
record = {'historical_red_source_sha256': red['candidate_source_tree_sha256'],
          'green_source_sha256': manifest['candidate_source_tree_sha256'],
          'historical_red_archive_sha256': sha(archive.read_bytes()),
          'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
          'unchanged_test_file_sha256': red_record['test_file_sha256'], 'changed_production_files': changed}
(OUT / 'green-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
