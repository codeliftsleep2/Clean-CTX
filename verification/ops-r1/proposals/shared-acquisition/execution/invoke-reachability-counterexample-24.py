"""Three fixed reachability controls; existing adapter enforces all child safeguards."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import hashlib

ROOT = Path(__file__).resolve().parents[5]
ISOLATED = ROOT / 'target/ops-r1/m3-isolated-23'
ADAPTER = ISOLATED / 'verification/ops-r1/proposals/shared-acquisition/execution/capture.py'
CASES = {
    'seed_is_not_sibling_disk_authority': 'dependency proof must be reused',
    'standalone_capture_and_missing_module_qualification': 'missing-module proof must be reused',
    'precached_attached_unit_replays_only_its_source': 'attached-unit control requires reuse',
}


def main():
    build = Path(sys.argv[1]).resolve()
    version, expectation = sys.argv[2:4]
    if expectation not in ('hardened-fail', 'legacy-pass'):
        raise ValueError('Unknown experiment expectation')
    out = ROOT / 'target/ops-r1' / ('reachability-24-' + expectation + '-' + time.strftime('%Y%m%d-%H%M%S'))
    out.mkdir(exist_ok=False)
    record = {'expectation': expectation, 'source_version': version,
              'build_receipt': str(build), 'runs': [], 'status': 'IN_PROGRESS'}
    save = lambda: (out / 'review.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    save()
    env = dict(os.environ, OPS_R1_STAGE='green', OPS_R1_GREEN_SOURCE=version)
    for case, marker in CASES.items():
        result = subprocess.run([sys.executable, str(ADAPTER), '--build-receipt', str(build),
                                 '--case', case, '--wall-seconds', '60', '--memory-mib', '512',
                                 '--stop-wait', '5'], cwd=ISOLATED, env=env, capture_output=True, text=True)
        (out / (case + '-adapter.txt')).write_text(result.stdout + result.stderr, encoding='utf-8')
        try:
            folder = Path(result.stdout.strip())
            if not folder.resolve().is_relative_to((ISOLATED / 'target/ops-r1').resolve()):
                raise ValueError('Invalid evidence directory')
            receipt = json.loads((folder / 'receipt.json').read_bytes())
            stdout = (folder / 'stdout.txt').read_text(encoding='utf-8')
            stderr = (folder / 'stderr.txt').read_text(encoding='utf-8')
            good = receipt['post_source_and_binary_identity_verified'] and 'running 1 test' in stdout
            good = good and receipt['command'][2].endswith('ops_r1_shared_' + case)
            if expectation == 'hardened-fail':
                good = (good and receipt['exit_code'] == 101 and result.returncode == 1
                        and receipt['status'] == 'TRACKED_TEST_FAILED_REVIEW_REASON'
                        and '0 passed; 1 failed; 0 ignored;' in stdout and marker in stderr
                        and 'left: [2, 2, 4]' in stderr and 'right: [1, 1, 2]' in stderr)
            else:
                good = (good and receipt['exit_code'] == 0 and result.returncode == 0
                        and receipt['status'] == 'TRACKED_TEST_COMPLETED'
                        and '1 passed; 0 failed; 0 ignored;' in stdout
                        and 'acquisition=[2, 2, 4]' in stderr)
            if (folder / 'watchdog-stop.json').exists():
                good = False
            record['runs'].append({'case': case, 'intended_outcome_observed': bool(good),
                                   'receipt': str(folder / 'receipt.json'), 'observations': receipt,
                                   'receipt_sha256': hashlib.sha256((folder / 'receipt.json').read_bytes()).hexdigest()})
            save()
            if not good:
                raise ValueError('Unexpected outcome; no later experiment launched')
            print(case + ': intended ' + expectation + ' observed', flush=True)
        except Exception as error:
            record.update(status='STOPPED_UNEXPECTED_RESULT', error=str(error))
            save()
            print(str(out), flush=True)
            return 1
    record['status'] = 'ALL_THREE_INTENDED_OUTCOMES_OBSERVED'
    save()
    print(str(out), flush=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
