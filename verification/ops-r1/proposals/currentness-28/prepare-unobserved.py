"""Diagnostic-only isolation of warm caller-thread observer bookkeeping."""
import hashlib
import json
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
ISO = ROOT / 'target/ops-r1/m3-isolated-23'
EXEC = ISO / 'verification/ops-r1/proposals/shared-acquisition/execution'
phase = sys.argv[1]
sha = lambda data: hashlib.sha256(data).hexdigest()
current_pin = EXEC / ('reviewed-green-currentness-28-' + ('performance' if phase == 'baseline' else 'unobserved-baseline') + '-sources.json')
manifest = json.loads(current_pin.read_bytes())
names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo'], cwd=ISO).decode().split('\0')
names = sorted(set(filter(None, names)) | {entry['path'] for entry in manifest['files']})


def identity():
    rows = []
    for name in names:
        data = (ISO / name).read_bytes()
        if Path(name).suffix in ('.rs', '.toml') or name == 'Cargo.lock':
            data = data.replace(b'\r\n', b'\n')
        rows.append({'path': name, 'sha256': sha(data)})
    return sha(json.dumps(rows, separators=(',', ':')).encode())


assert identity() == manifest['candidate_source_tree_sha256']
changed = ['src/ir/typescript_resolution/host.rs', 'src/workspace/index/coverage.rs',
           'src/workspace/index/resolver_registrations.rs', 'src/mcp/semantic_compilation/retained.rs']
if phase == 'baseline':
    build = json.loads((ISO / 'target/ops-r1/currentness-28-measurement-build-01/build-receipt.json').read_bytes())
    binary = Path(build['executable'])
    assert sha(binary.read_bytes()) == build['executable_sha256']
    preserved = ROOT / 'target/ops-r1/currentness-28-preserved-observed-candidate'
    preserved.mkdir(exist_ok=False)
    shutil.copy2(binary, preserved / binary.name)
    with zipfile.ZipFile(OUT / 'unobserved-candidate-producer.zip', 'x', compression=zipfile.ZIP_DEFLATED) as saved:
        for name in changed:
            saved.write(ISO / name, name)
    with zipfile.ZipFile(OUT / 'measurement-baseline-inputs.zip') as baseline:
        for name in changed:
            (ISO / name).write_bytes(baseline.read(name))
    observer = ISO / 'src/tests/architecture/work_observations_af_perf_002.rs'
    text = observer.read_text(encoding='utf-8')
    old = '    let identity = serde_json::to_string(value).expect("witness observation serialization");\n    update(|s| {\n'
    new = '    update(|s| {\n        let identity = serde_json::to_string(value).expect("witness observation serialization");\n'
    assert text.count(old) == 1
    observer.write_bytes(text.replace(old, new).encode('utf-8'))
    fixture = ISO / 'src/tests/mcp/ops_r1_absent_route.rs'
    text = fixture.read_text(encoding='utf-8')
    begin = text.index('        let (response, operations) = crate::ir::typescript_resolution::shared_acquisition::observe(')
    end = text.index('        let query_ms =', begin)
    dispatch = text[begin:end].replace('let (response, operations) =', 'let dispatch = ||', 1)
    dispatch += '        let (response, operations) = if request == 0 { dispatch() } else { crate::work_observations_af_perf_002::unobserved(dispatch) };\n'
    fixture.write_bytes((text[:begin] + dispatch + text[end:]).encode('utf-8'))
    subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', str(observer), str(fixture)], cwd=ISO, check=True)
elif phase == 'candidate':
    with zipfile.ZipFile(OUT / 'unobserved-candidate-producer.zip') as saved:
        for name in changed:
            (ISO / name).write_bytes(saved.read(name))
else:
    raise ValueError('Unknown measurement phase')

version = 'currentness-28-unobserved-' + phase
manifest['standing'] = 'Diagnostic warm caller-thread observer disabled; light OPS counters retained; cold worker instrumentation remains'
manifest['candidate_source_tree_sha256'] = identity()
pin = EXEC / ('reviewed-green-' + version + '-sources.json')
assert not pin.exists()
pin.write_bytes((json.dumps(manifest, indent=2) + '\n').encode('utf-8'))
text = (EXEC / 'capture.py').read_text(encoding='utf-8')
text = text.replace("SOURCE_PINS={", "SOURCE_PINS={'green-" + version + "': '" + sha(pin.read_bytes()) + "', ", 1)
text = text.replace('if amendment not in (', "if amendment not in ('" + version + "',", 1)
text = text.replace('    proposal=pathlib', "        if amendment=='" + version + "':source_key='green-" + version + "'\n    proposal=pathlib", 1)
text = text.replace('    core.CASES=', "    if source_key=='green-" + version + "':selected_cases=CASES_PERFORMANCE_25\n    core.CASES=", 1)
(EXEC / 'capture.py').write_bytes(text.encode('utf-8'))
for path in [pin, EXEC / 'capture.py', ISO / 'src/tests/architecture/work_observations_af_perf_002.rs', ISO / 'src/tests/mcp/ops_r1_absent_route.rs']:
    shutil.copy2(path, OUT / ('unobserved-' + phase + '-' + path.name))
record = {'phase': phase, 'source_sha256': manifest['candidate_source_tree_sha256'],
          'adapter_sha256': sha((EXEC / 'capture.py').read_bytes()), 'source_pin_sha256': sha(pin.read_bytes()),
          'scope': 'Measurement workspace only; root candidate unchanged; warm observer bookkeeping isolated'}
(OUT / ('unobserved-' + phase + '-preparation.json')).write_bytes((json.dumps(record, indent=2) + '\n').encode('utf-8'))
print(json.dumps(record, indent=2))
