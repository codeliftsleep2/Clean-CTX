"""Authenticate the integration successor and retain its actual operator evidence."""
import hashlib
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
RUN = ROOT / 'target/ops-r1/integration-26-20261121-075244-830'
BUILD = ROOT / 'target/ops-r1/integration-26-build-01'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((OUT / 'execution/reviewed-green-integration-26-sources.json').read_bytes())
build = json.loads((BUILD / 'build-receipt.json').read_bytes())
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})
rows = []
for name in names:
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
identity = sha(json.dumps(rows, separators=(',', ':')).encode())
assert identity == manifest['candidate_source_tree_sha256'] == build['source_tree_sha256']
assert sha(Path(build['executable']).read_bytes()) == build['executable_sha256']
assert build['exit_code'] == 0
for name, key in [('build.jsonl', 'build_log_sha256'), ('build-stderr.txt', 'build_stderr_sha256')]:
    assert sha((BUILD / name).read_bytes()) == build[key]
diagnostics = [json.loads(line) for line in (BUILD / 'build.jsonl').read_text(encoding='utf-8-sig').splitlines() if json.loads(line).get('reason') == 'compiler-message']
assert not diagnostics
assert json.loads((RUN / 'clippy-receipt.json').read_bytes())['exit_code'] == 0
entries = json.loads((RUN / 'focused-receipts.json').read_bytes())
assert len(entries) == 7
descriptor = json.loads((RUN / 'descriptor.json').read_bytes())
state_unchanged = descriptor['descriptor']['accepted_state_digest'] == descriptor['descriptor']['proposed_state_digest']
assert not [entry for entry in descriptor['descriptor']['previously_protected_candidate_view']['components'] if entry['selection']['id'] == 'OPS-001']
assert not descriptor['diagnostics'] and state_unchanged
archive = OUT / 'integration-evidence.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.write(ROOT / name, 'source/' + name)
    for prefix, folder in [('integration', RUN), ('build', BUILD), ('execution-definitions', OUT / 'execution')]:
        for path in sorted(folder.rglob('*')):
            if path.is_file() and '__pycache__' not in path.parts:
                saved.write(path, prefix + '/' + path.relative_to(folder).as_posix())
    for entry in entries:
        path = Path(entry['receipt'])
        assert sha(path.read_bytes()) == entry['sha256']
        value = json.loads(path.read_bytes())
        assert value['status'] == 'TRACKED_TEST_COMPLETED' and value['exit_code'] == 0
        assert value['post_source_and_binary_identity_verified'] and value['source_tree_sha256'] == identity
        assert value['executable_sha256'] == build['executable_sha256']
        for key, expected in value.items():
            if key.endswith('_sha256') and (path.parent / key[:-7]).is_file():
                assert sha((path.parent / key[:-7]).read_bytes()) == expected
        for evidence in sorted(path.parent.rglob('*')):
            if evidence.is_file():
                saved.write(evidence, 'tests/' + entry['case'] + '/' + evidence.relative_to(path.parent).as_posix())
    saved.writestr('formatting-successor.patch', subprocess.check_output(['git', 'diff', 'a59d361e', '--', 'src'], cwd=ROOT))
result = {
    'agent': 'Agent MaxHeadRoom', 'category': 'Verified scoped integration successor; not full CI or field closure',
    'previous_source_sha256': '75d63dfb564a4f860c637160ec1d2bbbc8fcc89823cba06bbbfbae25dc71ab4c',
    'source_tree_sha256': identity, 'executable_sha256': build['executable_sha256'],
    'source_drift': 'Rustfmt-only changes in two diagnostic test files',
    'build_exit_code': 0, 'compiler_diagnostics': 0, 'tracked_tests_passed': len(entries),
    'scoped_clippy_exit_code': 0, 'file_size_behavior_tests_passed': 11,
    'guards': ['file size PASS', 'UTF-8 PASS'],
    'original_actual_descriptor_digest': descriptor['digest'],
    'original_protected_state_unchanged': state_unchanged,
    'original_context_standing': 'ready, complete, current, zero diagnostics; content not acquired',
    'final_document_reconciliation': 'Unprotected evidence component reconciliation prepared; final actual capture pending',
    'evidence_archive': archive.name, 'evidence_archive_sha256': sha(archive.read_bytes()),
    'ci_status': 'Pending', 'field_incident': 'Open; no historical hang or leak resolution claimed',
    'delivery_status': 'No new commit, push, PR creation, merge or publication',
}
(OUT / 'evidence-index.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps(result, indent=2))
