"""Mission 24 focused sequence; existing adapter owns every bounded test child."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
SELECTION = HERE.parent / 'review-24/test-selections.json'
ADAPTER = HERE / 'capture.py'
BUILD = ROOT / sys.argv[1]
VERSION = sys.argv[2]
CASES = sys.argv[3:]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    build = json.loads(BUILD.read_bytes())
    if sha(ADAPTER) != build['launcher_sha256']:
        raise ValueError('Build/adapter pin mismatch')
    selection = json.loads(SELECTION.read_bytes())
    if CASES:
        selection = [next(item for item in selection if item['case'] == case) for case in CASES]
    if len({item['identity'] for item in selection}) != len(selection):
        raise ValueError('Duplicate selected identity')
    out = ROOT / 'target/ops-r1' / ('mission-24-' + VERSION + '-' + time.strftime('%Y%m%d-%H%M%S'))
    out.mkdir(exist_ok=False)
    record = {'standing': 'Mission24 exact tests through existing bounded adapter',
              'build_receipt': str(BUILD), 'build_receipt_sha256': sha(BUILD),
              'selection_sha256': sha(SELECTION), 'selection': selection, 'runs': [],
              'source_tree_sha256': build['source_tree_sha256'],
              'executable_sha256': build['executable_sha256'], 'status': 'IN_PROGRESS'}
    save = lambda: (out / 'bundle-review.json').write_text(
        json.dumps(record, indent=2) + '\n', encoding='utf-8')
    env = dict(os.environ, OPS_R1_STAGE='green', OPS_R1_GREEN_SOURCE=VERSION)
    save()
    try:
        for number, item in enumerate(selection, 1):
            print(f'[{number}/{len(selection)}] {item["case"]}', flush=True)
            run = subprocess.run([sys.executable, str(ADAPTER), '--build-receipt', str(BUILD),
                                  '--case', item['case'], '--wall-seconds', '60',
                                  '--memory-mib', '512', '--stop-wait', '5'],
                                 cwd=ROOT, env=env, capture_output=True, text=True)
            (out / f'{number:02d}-adapter-output.txt').write_text(
                run.stdout + run.stderr, encoding='utf-8')
            folder = Path(run.stdout.strip())
            if not folder.resolve().is_relative_to((ROOT / 'target/ops-r1').resolve()):
                raise ValueError('No valid evidence directory: ' + run.stdout + run.stderr)
            receipt = json.loads((folder / 'receipt.json').read_bytes())
            stdout = (folder / 'stdout.txt').read_text(encoding='utf-8')
            entry = {'case': item['case'], 'identity': item['identity'],
                     'receipt': str(folder / 'receipt.json'),
                     'receipt_sha256': sha(folder / 'receipt.json'), 'observations': receipt}
            record['runs'].append(entry)
            passed = (run.returncode == 0 and receipt['exit_code'] == 0
                      and receipt['status'] == 'TRACKED_TEST_COMPLETED'
                      and receipt['post_source_and_binary_identity_verified']
                      and receipt['command'][2] == item['identity']
                      and re.search(r'^running 1 test\s*$', stdout, re.M)
                      and 'test result: ok. 1 passed; 0 failed; 0 ignored;' in stdout)
            entry['passed'] = bool(passed)
            save()
            if not passed:
                raise ValueError('First failure: ' + str(folder))
            print('PASS: ' + str(folder), flush=True)
        record['status'] = 'ALL_SELECTED_PASSED'
    except Exception as error:
        record.update(status='STOPPED_AT_FIRST_FAILURE', error=str(error))
    save()
    print(str(out), flush=True)
    return 0 if record['status'] == 'ALL_SELECTED_PASSED' else 1


if __name__ == '__main__':
    sys.exit(main())
