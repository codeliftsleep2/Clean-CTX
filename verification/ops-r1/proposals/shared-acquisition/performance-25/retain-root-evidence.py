"""Authenticate and retain operator root verification; no builds or tests."""
import hashlib
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
OUT = Path(__file__).resolve().parent
RUN = ROOT / 'target/ops-r1/performance-root-verification-20261121-015331-278'
BUILD = ROOT / 'target/ops-r1/performance-25-root-build-01'
EXEC = OUT.parent / 'execution'
sha = lambda data: hashlib.sha256(data).hexdigest()
receipt = json.loads((BUILD / 'build-receipt.json').read_bytes())
manifest = json.loads((EXEC / 'reviewed-green-performance-25-root-sources.json').read_bytes())
tracked = subprocess.check_output(
    ['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'],
    cwd=ROOT).decode().split('\0')
rows = []
for name in sorted(set(filter(None, tracked)) | {p['path'] for p in manifest['files']}):
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
source = sha(json.dumps(rows, separators=(',', ':')).encode())
assert source == receipt['source_tree_sha256'] == manifest['candidate_source_tree_sha256']
assert sha(Path(receipt['executable']).read_bytes()) == receipt['executable_sha256']
assert sha((BUILD / 'build.jsonl').read_bytes()) == receipt['build_log_sha256']
assert sha((BUILD / 'build-stderr.txt').read_bytes()) == receipt['build_stderr_sha256']
assert sha((EXEC / 'capture.py').read_bytes()) == receipt['launcher_sha256']
messages = [json.loads(line) for line in (BUILD / 'build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
diagnostics = [message for message in messages if message.get('reason') == 'compiler-message']
assert not diagnostics and receipt['exit_code'] == 0
clippy = json.loads((RUN / 'clippy-receipt.json').read_bytes())
assert clippy['exit_code'] == 0
entries = json.loads((RUN / 'receipts.json').read_bytes())
assert len(entries) == 4
archive = OUT / 'root-verification-evidence.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for entry in entries:
        path = Path(entry['receipt'])
        assert sha(path.read_bytes()) == entry['sha256']
        observed = json.loads(path.read_bytes())
        assert observed['status'] == 'TRACKED_TEST_COMPLETED'
        assert observed['exit_code'] == 0 and observed['post_source_and_binary_identity_verified']
        assert observed['source_tree_sha256'] == source
        assert observed['executable_sha256'] == receipt['executable_sha256']
        for key, expected in observed.items():
            if key.endswith('_sha256') and (path.parent / key[:-7]).is_file():
                assert sha((path.parent / key[:-7]).read_bytes()) == expected
        for evidence in sorted(path.parent.rglob('*')):
            if evidence.is_file():
                saved.write(evidence, 'tests/' + entry['case'] + '/' + evidence.relative_to(path.parent).as_posix())
    for prefix, folder in [('build', BUILD), ('operator', RUN)]:
        for evidence in sorted(folder.iterdir()):
            if evidence.is_file():
                saved.write(evidence, prefix + '/' + evidence.name)
    for name in ['capture.py', 'invoke-build-capture.ps1', 'reviewed-green-performance-25-root-sources.json']:
        saved.write(EXEC / name, 'pins/' + name)
record = {
    'source_tree_sha256': source,
    'executable_sha256': receipt['executable_sha256'],
    'build_exit_code': receipt['exit_code'], 'compiler_diagnostics': len(diagnostics),
    'root_focused_tests_passed': len(entries),
    'scoped_clippy': clippy, 'scoped_clippy_log_sha256': sha((RUN / 'clippy.txt').read_bytes()),
    'source_archive': 'formatted-root-source.zip',
    'source_archive_sha256': sha((OUT / 'formatted-root-source.zip').read_bytes()),
    'verification_archive': archive.name, 'verification_archive_sha256': sha(archive.read_bytes()),
    'ci_status': 'Not executed or claimed', 'field_status': 'Historical incident remains open',
    'admission_status': 'Fresh actual admission required before publication; not claimed',
    'delivery_status': 'No commit, push, merge or publication',
}
(OUT / 'root-verification-results.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
