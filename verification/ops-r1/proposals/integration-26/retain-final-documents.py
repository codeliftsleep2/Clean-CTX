"""Verify exact final acquired source content and retain actual governance evidence."""
import hashlib
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
RUN = ROOT / 'target/ops-r1/integration-26-documents-20261121-091646-004'
sha = lambda data: hashlib.sha256(data).hexdigest()
descriptor = json.loads((RUN / 'descriptor.json').read_bytes())
assert not descriptor['diagnostics']
state = descriptor['descriptor']
assert state['accepted_state_digest'] == state['proposed_state_digest']
context = json.loads(next((RUN / 'context').glob('*.payload.json')).read_bytes())
assert context['result_state'] == 'ready' and context['complete'] and context['catalog_current']
assert not context['diagnostics']
requests = json.loads((RUN / 'acquire-requests.json').read_bytes())
ids = []
count = 0
for path in sorted((RUN / 'acquisition').glob('*.payload.json')):
    value = json.loads(path.read_bytes())
    assert value['result_state'] == 'ready' and value['complete'] and value['catalog_current']
    assert value['content_acquired'] and not value['diagnostics']
    assert value['generation'] == context['generation']
    for acquisition in value['acquisitions']:
        assert acquisition['complete'] and acquisition['result_state'] == 'ready'
        ids.append(acquisition['reference']['id'])
        for component in acquisition['components']:
            source = (ROOT / component['path']).read_bytes()
            assert sha(source) == component['source_sha256'], component['path']
            assert sha(component['text'].encode('utf-8')) == component['content_sha256']
            assert source[component['start']:component['end']] == component['text'].encode('utf-8')
            count += 1
assert len(requests) == 6
assert set(ids) == {'OPS-001', 'DOC-270', 'VERIFY-001', 'AUTH.FINAL_VERIFICATION', 'PERF-004-DOCTRINE', 'PERF-005-BOUNDED'}
manifest = json.loads((OUT / 'execution/reviewed-green-integration-26-sources.json').read_bytes())
tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
rows = []
for name in sorted(set(filter(None, tracked)) | {entry['path'] for entry in manifest['files']}):
    data = (ROOT / name).read_bytes()
    if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
        data = data.replace(b'\r\n', b'\n')
    rows.append({'path': name, 'sha256': sha(data)})
source_identity = sha(json.dumps(rows, separators=(',', ':')).encode())
assert source_identity == manifest['candidate_source_tree_sha256']
archive = OUT / 'final-document-evidence.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for path in sorted(RUN.rglob('*')):
        if path.is_file():
            saved.write(path, path.relative_to(RUN).as_posix())
result = {
    'agent': 'Agent MaxHeadRoom', 'category': 'Actual final checker and source acquisition verified',
    'source_tree_sha256': source_identity,
    'descriptor_digest': descriptor['digest'], 'context_generation': context['generation'],
    'descriptor_diagnostics': [], 'protected_state_unchanged': True,
    'context_ready_complete_current': True, 'source_acquisition_requests': len(requests),
    'acquired_components_verified': count, 'source_ids': sorted(set(ids)),
    'catalog_coverage': context['catalog_coverage'],
    'evidence_archive': archive.name, 'evidence_archive_sha256': sha(archive.read_bytes()),
    'authorization_submitted': False, 'publication_performed': False,
    'remaining_gates': ['Full CI on final integration commit', 'Owner-authorized delivery'],
    'original_field_incident': 'OPEN; no field hang or memory-retention resolution claimed',
}
(OUT / 'final-document-results.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps(result, indent=2))
