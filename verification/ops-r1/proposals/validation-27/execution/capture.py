"""Reviewed stage adapter for the existing OPS-R1 bounded monitor; never builds."""
import contextlib,hashlib,importlib.util,io,json,os,pathlib,sys
ROOT=pathlib.Path(__file__).resolve().parents[5]
SOURCE_PINS={'green-validation-27-green': '2003ee64eb8877ec0857e74e9e8188c3492a74f84f2370fcd28bfe0b98ee9e83', 'green-validation-27-red': '5c66552c58436ab83b940d6f44439877a53c12ae9a12416dfe96266086a407d0', 'green-integration-26': '23d9c6570455d7ac23f8429200f76d21b6f37997f5ad3185db41d43d92d74cba', 'green-performance-25-root': 'f4d5434f7c571070fad63e1cd4ce4d24cc28b11440bd87341d2c5095ef2d0756', 'red': 'ca0aba6c734d91da73be356a8c74779816ca8af254e43a63d6fe7e529bd51e14', 'green': '389a714aa2d037e73e92a623564c4198896bf2e5a6ccc2c00408381899f24ac5', 'green-amendment-18': '21734cd76b0d8a2a008b07964d31238b39354dce75af1456059ff71862d567a0', 'green-hardening-24b': '5aaf7517bd808cc56a18e8585592652cf9a45f71eb3ea5e236ff42730330d8a8', 'green-hardening-24c': '1355f65fdfab0e1c695b12d113e773aad839681f821e487f8becdb50618e5feb', 'green-hardening-24d': 'c0a1ff040ecb3d2d9825fc844e29d4c102abf6f7bef58caee3c7b37c6b90dd7f', 'green-hardening-24e': '661ae0180d4712cd6c620883a3a0650d013c0d768cf32472c00a1b78c299bf59', 'green-hardening-24': '5dd3b461e75ea90210a2120251be18d63ea2a24f522ed1b66db9e64550bc55e6'}
CASES={
    "fitness": [
        4,
        0
    ],
    "precached_host_unit_project_and_owner_only_proof": [
        1,
        1
    ],
    "seed_is_not_sibling_disk_authority": [
        1,
        2
    ],
    "divergent_overlay_and_private_cache_are_not_overwritten": [
        1,
        3
    ],
    "failed_partial_and_poisoned_leader_do_not_grant_or_poison": [
        1,
        4
    ],
    "standalone_capture_and_missing_module_qualification": [
        1,
        5
    ],
    "precached_attached_unit_replays_only_its_source": [
        1,
        6
    ],
    "failed_partial_after_one_unit_keeps_local_failure": [
        1,
        7
    ],
    "precached_unaccounted_values_never_seal_from_nonempty_witnesses": [
        1,
        8
    ],
    "config_roots_and_route_changes_stay_independent": [
        1,
        9
    ],
    "source_and_package_changes_invalidate_proof": [
        1,
        10
    ],
    "request_directory_and_inherited_project_remain_authoritative": [
        1,
        11
    ],
    "multiple_projects_and_operations_do_not_share_authority": [
        1,
        12
    ],
    "same_config_different_automatic_type_lookup_stays_distinct": [
        1,
        13
    ],
    "concurrent_owners_and_nested_acquisition_are_isolated": [
        1,
        14
    ],
    "bootstrap": [
        1,
        15
    ],
    "hydration_and_explicit": [
        1,
        16
    ],
    "worker_order_non_ts": [
        1,
        17
    ],
    "worker_failure_non_ts": [
        1,
        18
    ],
    "batch_prepublication": [
        1,
        19
    ],
    "typescript_prepublication": [
        1,
        20
    ],
    "typescript_persistence": [
        1,
        21
    ],
    "resolver_positive": [
        1,
        22
    ],
    "retirement": [
        1,
        23
    ],
    "csharp": [
        1,
        24
    ],
    "ambient_dependency": [
        1,
        25
    ]
}
CASES.update({'retirement_confirmed': [1, 26], 'retirement_denial': [1, 27], 'retirement_wrong_kind': [1, 28], 'retirement_history': [1, 29], 'recovery_refusal': [1, 30], 'hydration_refusal': [1, 31]})

CASES_HARDENING_24C={'fitness': [4, 0], 'precached_host_unit_project_and_owner_only_proof': [1, 1], 'seed_is_not_sibling_disk_authority': [1, 2], 'divergent_overlay_and_private_cache_are_not_overwritten': [1, 3], 'failed_partial_and_poisoned_leader_do_not_grant_or_poison': [1, 4], 'standalone_capture_and_missing_module_qualification': [1, 5], 'precached_attached_unit_replays_only_its_source': [1, 6], 'failed_partial_after_one_unit_keeps_local_failure': [1, 7], 'precached_unaccounted_values_never_seal_from_nonempty_witnesses': [1, 8], 'config_roots_and_route_changes_stay_independent': [1, 9], 'source_and_package_changes_invalidate_proof': [1, 10], 'request_directory_and_inherited_project_remain_authoritative': [1, 11], 'multiple_projects_and_operations_do_not_share_authority': [1, 12], 'same_config_different_automatic_type_lookup_stays_distinct': [1, 13], 'concurrent_owners_and_nested_acquisition_are_isolated': [1, 14], 'bootstrap': [1, 15], 'hydration_and_explicit': [1, 16], 'worker_order_non_ts': [1, 17], 'worker_failure_non_ts': [1, 18], 'batch_prepublication': [1, 19], 'typescript_prepublication': [1, 20], 'typescript_persistence': [1, 21], 'resolver_positive': [1, 22], 'resolver_positive_barrel': [1, 23], 'resolver_positive_namespace': [1, 24], 'resolver_positive_local': [1, 25], 'retirement': [1, 26], 'csharp': [1, 27], 'ambient_dependency': [1, 28], 'retirement_confirmed': [1, 29], 'retirement_denial': [1, 30], 'retirement_wrong_kind': [1, 31], 'retirement_history': [1, 32], 'recovery_refusal': [1, 33], 'hydration_refusal': [1, 34]}

CASES_HARDENING_24D={'fitness': [4, 0], 'precached_host_unit_project_and_owner_only_proof': [1, 1], 'seed_is_not_sibling_disk_authority': [1, 2], 'divergent_overlay_and_private_cache_are_not_overwritten': [1, 3], 'failed_partial_and_poisoned_leader_do_not_grant_or_poison': [1, 4], 'standalone_capture_and_missing_module_qualification': [1, 5], 'precached_attached_unit_replays_only_its_source': [1, 6], 'failed_partial_after_one_unit_keeps_local_failure': [1, 7], 'precached_unaccounted_values_never_seal_from_nonempty_witnesses': [1, 8], 'config_roots_and_route_changes_stay_independent': [1, 9], 'source_and_package_changes_invalidate_proof': [1, 10], 'request_directory_and_inherited_project_remain_authoritative': [1, 11], 'multiple_projects_and_operations_do_not_share_authority': [1, 12], 'same_config_different_automatic_type_lookup_stays_distinct': [1, 13], 'concurrent_owners_and_nested_acquisition_are_isolated': [1, 14], 'bootstrap': [1, 15], 'hydration_and_explicit': [1, 16], 'worker_order_non_ts': [1, 17], 'worker_failure_non_ts': [1, 18], 'batch_prepublication': [1, 19], 'typescript_prepublication': [1, 20], 'typescript_persistence': [1, 21], 'resolver_positive': [1, 22], 'resolver_positive_barrel': [1, 23], 'resolver_positive_namespace': [1, 24], 'resolver_positive_local': [1, 25], 'retirement': [1, 26], 'csharp': [1, 27], 'ambient_dependency': [1, 28], 'retirement_confirmed': [1, 29], 'retirement_denial': [1, 30], 'retirement_wrong_kind': [1, 31], 'retirement_history': [1, 32], 'recovery_refusal': [1, 33], 'hydration_refusal': [1, 34], 'csharp_preparation_failure': [1, 35]}

def digest(data):return hashlib.sha256(data).hexdigest()
def main():
    stage=os.environ.get('OPS_R1_STAGE')
    if stage not in SOURCE_PINS:raise ValueError('Explicit OPS_R1_STAGE=red or green required')
    source_key=stage
    source_name='reviewed-red-sources.json'
    if stage=='green':
        amendment=os.environ.get('OPS_R1_GREEN_SOURCE','amendment-18')
        if amendment not in ('amendment-15','amendment-18','hardening-24','hardening-24b','hardening-24c','hardening-24d','hardening-24e','performance-25-root','integration-26','validation-27-red','validation-27-green'):
            raise ValueError('Unknown pinned GREEN source amendment')
        source_name='reviewed-green-'+amendment+'-sources.json'
        if amendment=='amendment-18':source_key='green-amendment-18'
        if amendment=='hardening-24':source_key='green-hardening-24'
        if amendment=='hardening-24b':source_key='green-hardening-24b'
        if amendment=='hardening-24c':source_key='green-hardening-24c'
        if amendment=='hardening-24d':source_key='green-hardening-24d'
        if amendment=='hardening-24e':source_key='green-hardening-24e'
        if amendment=='performance-25-root':source_key='green-performance-25-root'
        if amendment=='integration-26':source_key='green-integration-26'
        if amendment=='validation-27-red':source_key='green-validation-27-red'
        if amendment=='validation-27-green':source_key='green-validation-27-green'
    proposal=pathlib.Path(__file__).with_name(source_name)
    if digest(proposal.read_bytes())!=SOURCE_PINS[source_key]:raise ValueError('Reviewed stage source pin changed')
    import subprocess
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    if head!='dbf495517db50cc83fab747c34b33ff989704e27':raise ValueError('Reviewed baseline HEAD changed')
    original=ROOT/'verification/ops-r1/p0/run_bounded.py'
    if digest(original.read_bytes())!='5acc4590c602ee2528c27419b446443fc1325664457adda0e1aab1639dec4484':
        raise ValueError('Reviewed monitor changed; no child authorized')
    spec=importlib.util.spec_from_file_location('ops_reviewed_monitor',original)
    core=importlib.util.module_from_spec(spec);spec.loader.exec_module(core)
    core.__file__=__file__
    core.PROPOSAL=proposal
    selected_cases=CASES_HARDENING_24C if source_key=='green-hardening-24c' else CASES
    if source_key in ('green-hardening-24d','green-hardening-24e'):selected_cases=CASES_HARDENING_24D
    if source_key in ('green-performance-25-root','green-integration-26'):selected_cases=dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37)})
    if source_key=='green-validation-27-red':selected_cases={'seal':(1,38),'qualification':(1,39)}
    if source_key=='green-validation-27-green':selected_cases=dict(CASES_HARDENING_24D, **{'cost-constraints':(1,36),'directory-work':(1,37),'seal':(1,38),'qualification':(1,39)})
    core.CASES={'fitness':(4,0)} if stage=='red' else {k:tuple(v) for k,v in selected_cases.items()}
    capture=io.StringIO()
    with contextlib.redirect_stdout(capture):code=core.main()
    output=capture.getvalue().strip()
    if '--verify-sources-only' in sys.argv or '--verify-baseline-only' in sys.argv:
        print(output);return code
    folder=pathlib.Path(output)
    if not folder.resolve().is_relative_to((ROOT/'target/ops-r1').resolve()):raise ValueError('Unexpected evidence destination')
    receipt_path=folder/'receipt.json';original_receipt=receipt_path.read_bytes();receipt=json.loads(original_receipt)
    (folder/'monitor-receipt.json').write_bytes(original_receipt)
    receipt['monitor_receipt_sha256']=digest(original_receipt)
    receipt['monitor_case_parameters']={key:receipt.pop(key) for key in ['units','component']}
    receipt['fixture_scope']='Four-owner public fitness' if receipt['case']=='fitness' else 'Pinned tracked control; fixture sizes are defined by that test, not the monitor compatibility parameters'
    receipt.update(proposal_stage=stage,source_pin_sha256=SOURCE_PINS[source_key],adapter_sha256=digest(pathlib.Path(__file__).read_bytes()),monitor_sha256=digest(original.read_bytes()))
    for name in ['work-counts.json']:
        if (folder/name).exists():receipt[name+'_sha256']=digest((folder/name).read_bytes())
    receipt['selection_standing']='One exact Rust tracked test; no CI admission or leak certification'
    receipt_path.write_text(json.dumps(receipt,indent=2)+chr(10),encoding='utf-8')
    print(folder);return code
if __name__=='__main__':sys.exit(main())
