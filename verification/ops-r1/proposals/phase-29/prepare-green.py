"""Preserve public work RED and pin consumption-only batch validation."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
VERSION = 'phase-29-green'
sha = lambda data: hashlib.sha256(data).hexdigest()
red_path = EXEC / 'reviewed-green-phase-29-red-sources.json'
red = json.loads(red_path.read_bytes())
changed = ['src/workspace/index/coverage.rs', 'src/mcp/tool_handlers/query/resolver_registrations.rs',
           'src/tests/ir/ops_r1_shared_acquisition_controls.rs']
added = 'src/tests/ir/ops_r1_phase_validation.rs'
with zipfile.ZipFile(ROOT / 'verification/ops-r1/proposals/currentness-28/validation-and-measurement-evidence.zip') as history:
    originals = {name: history.read('root-source/' + name) for name in changed}
tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
names = sorted(set(filter(None, tracked)) | {entry['path'] for entry in red['files']})


def identity(paths, baseline=False):
    rows = []
    for name in paths:
        data = originals[name] if baseline and name in originals else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity(names, True) == red['candidate_source_tree_sha256']
build_path = ROOT / 'target/ops-r1/phase-29-red-build-01/build-receipt.json'
build = json.loads(build_path.read_bytes())
binary = Path(build['executable'])
assert sha(binary.read_bytes()) == build['executable_sha256']
preserved = ROOT / 'target/ops-r1/phase-29-preserved-red'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build_path, preserved / 'build-receipt.json')
archive = OUT / 'historical-red.zip'
with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as saved:
    for name in names:
        saved.writestr('source/' + name, originals[name] if name in originals else (ROOT / name).read_bytes())
    for case, folder, expected in [('public-work', 'p0-public-work-20261123-093202', 101), ('phase-edit', 'p0-phase-edit-20261123-093159', 0)]:
        capture = ROOT / 'target/ops-r1' / folder
        receipt = json.loads((capture / 'receipt.json').read_bytes())
        assert receipt['exit_code'] == expected and receipt['post_source_and_binary_identity_verified']
        assert receipt['source_tree_sha256'] == red['candidate_source_tree_sha256']
        for path in sorted(capture.rglob('*')):
            if path.is_file():
                saved.write(path, case + '/' + path.relative_to(capture).as_posix())
    for path in [red_path, EXEC / 'capture.py', EXEC / 'invoke-build-capture.ps1', build_path]:
        saved.write(path, 'pins/' + path.name)
manifest = dict(red)
offset = len(manifest['test_identities'])
manifest['files'] += [{'path': added}]
manifest['test_identities'] += ['ir::typescript_resolution::ops_r1_shared_acquisition_controls::phase_validation::ops_r1_consumption_batch_refuses_conflicting_config_hashes_and_missing_owners']
manifest['standing'] = 'Exact witness sharing within the existing consumption call; preparation and direct calls remain fresh'
manifest['candidate_source_tree_sha256'] = identity(sorted(set(names) | {added}))
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace('if amendment not in (', "if amendment not in ('" + VERSION + "',", 1)
text = text.replace('    proposal=pathlib', "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace('    core.CASES=', "    if source_key=='green-" + VERSION + "':selected_cases=dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37),'seal':(1,38),'qualification':(1,39),'retained-currentness':(1,40),'public-work':(1,41),'phase-edit':(1,42),'conflicting-facts':(1," + str(offset) + ")})\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
record = {'red_source_sha256': red['candidate_source_tree_sha256'], 'green_source_sha256': manifest['candidate_source_tree_sha256'],
          'historical_red_archive_sha256': sha(archive.read_bytes()), 'source_pin_sha256': sha(pin.read_bytes()),
          'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
          'public_work_witness_sha256': sha((ROOT / 'src/tests/mcp/ops_r1_acquisition_fitness.rs').read_bytes()),
          'phase_edit_witness_sha256': sha((ROOT / 'src/tests/mcp/ops_r1_query_phase_currentness.rs').read_bytes()),
          'production_changed': changed[:2], 'preparation_unchanged': True}
(OUT / 'green-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
