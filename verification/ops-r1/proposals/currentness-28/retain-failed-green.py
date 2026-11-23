"""Preserve the original fixture failure before correcting its currentness premise."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
BUILD = ROOT / 'target/ops-r1/currentness-28-green-build-01'
FAILURE = ROOT / 'target/ops-r1/p0-retained-currentness-20261122-094512'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((EXEC / 'reviewed-green-currentness-28-green-sources.json').read_bytes())
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})
rows = []
for name in names:
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
assert sha(json.dumps(rows, separators=(',', ':')).encode()) == manifest['candidate_source_tree_sha256']
build = json.loads((BUILD / 'build-receipt.json').read_bytes())
binary = Path(build['executable'])
assert sha(binary.read_bytes()) == build['executable_sha256']
preserved = ROOT / 'target/ops-r1/currentness-28-preserved-failed-green'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(BUILD / 'build-receipt.json', preserved / 'build-receipt.json')
archive = OUT / 'failed-green-original-fixture.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.write(ROOT / name, 'source/' + name)
    for prefix, folder in [('build', BUILD), ('execution', FAILURE), ('definitions', EXEC)]:
        for path in sorted(folder.rglob('*')):
            if path.is_file() and '__pycache__' not in path.parts:
                saved.write(path, prefix + '/' + path.relative_to(folder).as_posix())
record = {'source_sha256': manifest['candidate_source_tree_sha256'],
          'executable_sha256': build['executable_sha256'], 'archive_sha256': sha(archive.read_bytes()),
          'classification': 'Original duplicate-work assertion passed; subsequent fixture used stale caller owner hash after editing supplied owner source',
          'original_failure_preserved': True, 'remaining_controls_unexecuted': 14}
(OUT / 'failed-green-original-fixture.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
