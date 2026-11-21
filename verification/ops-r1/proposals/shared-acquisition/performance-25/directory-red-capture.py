"""Reviewed stage adapter for the existing OPS-R1 bounded monitor; never builds."""
import contextlib,hashlib,importlib.util,io,json,os,pathlib,sys
ROOT=pathlib.Path(__file__).resolve().parents[5]
SOURCE_PINS={'green-performance-25-directory-red': 'c6d4a3bb4b0344d9a400a623a43a7c2102b0d3e0a938c27d1d8a62ab18c70248', 'green-performance-25-cost-green': 'c005ebe338af98b28cebb31fbc2d7f3b964973e8820a1c43fa7190fe6d8760c9', 'green-performance-25-cost-red': 'a51681b2a426c4a5a7f7c7c8b17e1817fac6b5760899ecdc07dcd1a579dd81fb', 'red': 'ca0aba6c734d91da73be356a8c74779816ca8af254e43a63d6fe7e529bd51e14', 'green': '389a714aa2d037e73e92a623564c4198896bf2e5a6ccc2c00408381899f24ac5', 'green-amendment-18': '21734cd76b0d8a2a008b07964d31238b39354dce75af1456059ff71862d567a0', 'green-hardening-24b': '5aaf7517bd808cc56a18e8585592652cf9a45f71eb3ea5e236ff42730330d8a8', 'green-hardening-24c': '1355f65fdfab0e1c695b12d113e773aad839681f821e487f8becdb50618e5feb', 'green-hardening-24d': 'c0a1ff040ecb3d2d9825fc844e29d4c102abf6f7bef58caee3c7b37c6b90dd7f', 'green-hardening-24e': '661ae0180d4712cd6c620883a3a0650d013c0d768cf32472c00a1b78c299bf59', 'green-performance-25-baseline': '2230fa0d8e6df68cd7c40b2922f842fb4187ad79b9522622bf75c36f13e0fc91', 'green-performance-25-baselineb': 'b0023d56fbdb035888f858cf173670fb00a6ecfe064ebcacfad68e597b0dce30', 'green-hardening-24': '5dd3b461e75ea90210a2120251be18d63ea2a24f522ed1b66db9e64550bc55e6'}
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

CASES_PERFORMANCE_25={'fitness': [4, 0], 'precached_host_unit_project_and_owner_only_proof': [1, 1], 'seed_is_not_sibling_disk_authority': [1, 2], 'divergent_overlay_and_private_cache_are_not_overwritten': [1, 3], 'failed_partial_and_poisoned_leader_do_not_grant_or_poison': [1, 4], 'standalone_capture_and_missing_module_qualification': [1, 5], 'precached_attached_unit_replays_only_its_source': [1, 6], 'failed_partial_after_one_unit_keeps_local_failure': [1, 7], 'precached_unaccounted_values_never_seal_from_nonempty_witnesses': [1, 8], 'config_roots_and_route_changes_stay_independent': [1, 9], 'source_and_package_changes_invalidate_proof': [1, 10], 'request_directory_and_inherited_project_remain_authoritative': [1, 11], 'multiple_projects_and_operations_do_not_share_authority': [1, 12], 'same_config_different_automatic_type_lookup_stays_distinct': [1, 13], 'concurrent_owners_and_nested_acquisition_are_isolated': [1, 14], 'bootstrap': [1, 15], 'hydration_and_explicit': [1, 16], 'worker_order_non_ts': [1, 17], 'worker_failure_non_ts': [1, 18], 'batch_prepublication': [1, 19], 'typescript_prepublication': [1, 20], 'typescript_persistence': [1, 21], 'resolver_positive': [1, 22], 'resolver_positive_barrel': [1, 23], 'resolver_positive_namespace': [1, 24], 'resolver_positive_local': [1, 25], 'retirement': [1, 26], 'csharp': [1, 27], 'ambient_dependency': [1, 28], 'retirement_confirmed': [1, 29], 'retirement_denial': [1, 30], 'retirement_wrong_kind': [1, 31], 'retirement_history': [1, 32], 'recovery_refusal': [1, 33], 'hydration_refusal': [1, 34], 'csharp_preparation_failure': [1, 35], 'flat-scripts-1': [1, 36], 'flat-modules-1': [1, 36], 'directories-scripts-1': [1, 36], 'directories-modules-1': [1, 36], 'fixed-1': [1, 37], 'flat-scripts-4': [4, 36], 'flat-modules-4': [4, 36], 'directories-scripts-4': [4, 36], 'directories-modules-4': [4, 36], 'fixed-4': [4, 37], 'flat-scripts-16': [16, 36], 'flat-modules-16': [16, 36], 'directories-scripts-16': [16, 36], 'directories-modules-16': [16, 36], 'fixed-16': [16, 37], 'flat-scripts-64': [64, 36], 'flat-modules-64': [64, 36], 'directories-scripts-64': [64, 36], 'directories-modules-64': [64, 36], 'fixed-64': [64, 37]}

def digest(data):return hashlib.sha256(data).hexdigest()
def main():
    stage=os.environ.get('OPS_R1_STAGE')
    if stage not in SOURCE_PINS:raise ValueError('Explicit OPS_R1_STAGE=red or green required')
    source_key=stage
    source_name='reviewed-red-sources.json'
    if stage=='green':
        amendment=os.environ.get('OPS_R1_GREEN_SOURCE','amendment-18')
        if amendment not in ('amendment-15','amendment-18','hardening-24','hardening-24b','hardening-24c','hardening-24d','hardening-24e','performance-25-baseline','performance-25-baselineb','performance-25-cost-red','performance-25-cost-green','performance-25-directory-red'):
            raise ValueError('Unknown pinned GREEN source amendment')
        source_name='reviewed-green-'+amendment+'-sources.json'
        if amendment=='amendment-18':source_key='green-amendment-18'
        if amendment=='hardening-24':source_key='green-hardening-24'
        if amendment=='hardening-24b':source_key='green-hardening-24b'
        if amendment=='hardening-24c':source_key='green-hardening-24c'
        if amendment=='hardening-24d':source_key='green-hardening-24d'
        if amendment=='hardening-24e':source_key='green-hardening-24e'
        if amendment=='performance-25-baseline':source_key='green-performance-25-baseline'
        if amendment=='performance-25-baselineb':source_key='green-performance-25-baselineb'
        if amendment=='performance-25-cost-red':source_key='green-performance-25-cost-red'
        if amendment=='performance-25-cost-green':source_key='green-performance-25-cost-green'
        if amendment=='performance-25-directory-red':source_key='green-performance-25-directory-red'
    proposal=pathlib.Path(__file__).with_name(source_name)
    if digest(proposal.read_bytes())!=SOURCE_PINS[source_key]:raise ValueError('Reviewed stage source pin changed')
    import subprocess
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    if head!='0dd8d7503af3116241ad7c1e23c293b1d5849cc1':raise ValueError('Reviewed baseline HEAD changed')
    original=ROOT/'verification/ops-r1/p0/run_bounded.py'
    if digest(original.read_bytes())!='5acc4590c602ee2528c27419b446443fc1325664457adda0e1aab1639dec4484':
        raise ValueError('Reviewed monitor changed; no child authorized')
    spec=importlib.util.spec_from_file_location('ops_reviewed_monitor',original)
    core=importlib.util.module_from_spec(spec);spec.loader.exec_module(core)
    core.__file__=__file__
    core.PROPOSAL=proposal
    selected_cases=CASES_HARDENING_24C if source_key=='green-hardening-24c' else CASES
    if source_key in ('green-hardening-24d','green-hardening-24e'):selected_cases=CASES_HARDENING_24D
    if source_key in ('green-performance-25-baseline','green-performance-25-baselineb'):selected_cases=CASES_PERFORMANCE_25
    if source_key=='green-performance-25-cost-red':selected_cases={'cost-constraints':(1,38)}
    if source_key=='green-performance-25-cost-green':selected_cases=dict(CASES_PERFORMANCE_25, **{'cost-constraints':(1,38)})
    if source_key=='green-performance-25-directory-red':selected_cases={'directory-work':(1,39)}
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
