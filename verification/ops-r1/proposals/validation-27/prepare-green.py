"""Retain intended REDs and pin the per-call exact-constraint correction."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
VERSION = 'validation-27-green'
sha = lambda value: hashlib.sha256(value).hexdigest()
red_path = EXEC / 'reviewed-green-validation-27-red-sources.json'
red = json.loads(red_path.read_bytes())
before = json.loads((OUT / 'red-preparation.json').read_bytes())
changed = 'src/ir/typescript_resolution/shared_acquisition.rs'
test = 'src/tests/ir/ops_r1_shared_acquisition_cost.rs'
assert sha((ROOT / test).read_bytes()) == before['new_test_file_sha256'], 'RED assertions must remain exact'
with zipfile.ZipFile(ROOT / 'verification/ops-r1/proposals/integration-26/integration-evidence.zip') as prior:
    original = prior.read('source/' + changed)
tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {row['path'] for row in red['files']})


def identity(baseline):
    rows = []
    for name in names:
        data = original if baseline and name == changed else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity(True) == red['candidate_source_tree_sha256']
build_path = ROOT / 'target/ops-r1/validation-27-red-build-01/build-receipt.json'
build = json.loads(build_path.read_bytes())
binary = Path(build['executable'])
assert sha(binary.read_bytes()) == build['executable_sha256']
preserved = ROOT / 'target/ops-r1/validation-27-preserved-red'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build_path, preserved / 'build-receipt.json')
archive = OUT / 'historical-red.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.writestr('source/' + name, original if name == changed else (ROOT / name).read_bytes())
    for case, folder in [('seal', 'p0-seal-20261121-102414'), ('qualification', 'p0-qualification-20261121-102417')]:
        directory = ROOT / 'target/ops-r1' / folder
        receipt = json.loads((directory / 'receipt.json').read_bytes())
        assert receipt['exit_code'] == 101 and receipt['post_source_and_binary_identity_verified']
        assert receipt['source_tree_sha256'] == red['candidate_source_tree_sha256']
        assert receipt['executable_sha256'] == build['executable_sha256']
        for path in sorted(directory.rglob('*')):
            if path.is_file():
                saved.write(path, 'executions/' + case + '/' + path.relative_to(directory).as_posix())
    for path in [red_path, EXEC / 'capture.py', EXEC / 'invoke-build-capture.ps1', build_path]:
        saved.write(path, 'pins/' + path.name)
manifest = dict(red)
manifest['standing'] = 'Fresh per-call exact-constraint checks; source attribution and boundary freshness unchanged'
manifest['candidate_source_tree_sha256'] = identity(False)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace("'validation-27-red'):", "'validation-27-red','" + VERSION + "'):", 1)
text = text.replace("    proposal=pathlib", "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace("    core.CASES=", "    if source_key=='green-" + VERSION + "':selected_cases=dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37),'seal':(1,38),'qualification':(1,39)})\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
record = {'historical_red_source_sha256': red['candidate_source_tree_sha256'],
          'green_source_sha256': manifest['candidate_source_tree_sha256'],
          'historical_red_archive_sha256': sha(archive.read_bytes()),
          'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
          'unchanged_test_file_sha256': before['new_test_file_sha256'],
          'preserved_red_binary_sha256': build['executable_sha256'], 'changed_production_file': changed}
(OUT / 'green-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
