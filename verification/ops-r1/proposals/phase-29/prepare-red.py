"""Pin public consumption work and preparation/consumption separation witnesses."""
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
EXEC = OUT / 'execution'
BASE = ROOT / 'verification/ops-r1/proposals/currentness-28/execution'
VERSION = 'phase-29-red'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((BASE / 'reviewed-green-currentness-28-corrected-green-sources.json').read_bytes())
changed = ['src/mcp/tool_handlers/query.rs', 'src/tests/mcp/ops_r1_acquisition_fitness.rs']
added = 'src/tests/mcp/ops_r1_query_phase_currentness.rs'
with zipfile.ZipFile(ROOT / 'verification/ops-r1/proposals/currentness-28/validation-and-measurement-evidence.zip') as history:
    originals = {name: history.read('root-source/' + name) for name in changed}
tracked = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ROOT).decode().split('\0')
baseline_names = sorted(set(filter(None, tracked)) | {entry['path'] for entry in manifest['files']})


def identity(names, baseline=False):
    rows = []
    for name in names:
        data = originals[name] if baseline and name in originals else (ROOT / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


baseline = identity(baseline_names, True)
assert baseline == manifest['candidate_source_tree_sha256']
build_path = ROOT / 'target/ops-r1/currentness-28-corrected-green-build-01/build-receipt.json'
build = json.loads(build_path.read_bytes())
binary = Path(build['executable'])
assert build['source_tree_sha256'] == baseline and sha(binary.read_bytes()) == build['executable_sha256']
preserved = ROOT / 'target/ops-r1/phase-29-preserved-baseline'
preserved.mkdir(exist_ok=False)
shutil.copy2(binary, preserved / binary.name)
shutil.copy2(build_path, preserved / 'build-receipt.json')
offset = len(manifest['test_identities'])
manifest['files'] += [{'path': added}]
manifest['test_identities'] += [
    'mcp::semantic_compilation::ops_r1_acquisition_fitness::ops_r1_warm_resolver_consumption_checks_shared_constraints_once',
    'mcp::tool_handlers::query::ops_r1_query_phase_currentness::ops_r1_resolver_consumption_refuses_config_edit_after_preparation',
]
manifest['standing'] = 'Test-only public work RED plus independent preparation/consumption freshness control'
manifest['baseline_source_tree_sha256'] = baseline
manifest['candidate_source_tree_sha256'] = identity(sorted(set(baseline_names) | {added}))
EXEC.mkdir(exist_ok=True)
pin = EXEC / ('reviewed-green-' + VERSION + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (BASE / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + VERSION + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace('if amendment not in (', "if amendment not in ('" + VERSION + "',", 1)
text = text.replace('    proposal=pathlib', "        if amendment=='" + VERSION + "':source_key='green-" + VERSION + "'\n    proposal=pathlib", 1)
text = text.replace('    core.CASES=', "    if source_key=='green-" + VERSION + "':selected_cases={'public-work':(1," + str(offset) + "),'phase-edit':(1," + str(offset + 1) + ")}\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
(EXEC / 'invoke-build-capture.ps1').write_bytes((BASE / 'invoke-build-capture.ps1').read_bytes())
record = {'baseline_source_sha256': baseline, 'red_source_sha256': manifest['candidate_source_tree_sha256'],
          'source_pin_sha256': sha(pin.read_bytes()), 'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()),
          'production_consumer_unchanged': True, 'new_tests': manifest['test_identities'][-2:]}
(OUT / 'red-preparation.json').write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
