//! Actual integrated scan costs; default scan retains original construction.
use super::{Mode, capture, reset, snapshot, state};
use serde_json::json;
use std::path::PathBuf;

fn run(roots: &[PathBuf], proof: bool, original: bool, observe: bool) -> (u128, serde_json::Value) {
    let state = state(roots);
    let _mode = Mode::new(false, observe);
    capture(proof);
    reset();
    let selected = super::super::super::configured_roots(&state, Some(&roots[0].to_string_lossy()));
    let started = std::time::Instant::now();
    let scan = if original {
        super::super::super::scan(&state, &selected, "AbsentLocal")
    } else {
        super::super::super::scan_with_manifest_demand(&state, &selected, "AbsentLocal", proof)
    };
    let provisional = if proof {
        super::super::super::super::acquisition::prepare_clean_acquisition(
            crate::mcp::acquisition::AcquisitionTarget::declarations("Wanted0_0"),
            &scan.inventories,
            &state.config.exclude_patterns,
        )
    } else {
        None
    };
    let identity = if observe {
        json!({"candidates":scan.candidates,"attempted":scan.attempted,"complete":scan.completed,
            "work":snapshot(),"provisional":provisional.is_some()})
    } else {
        serde_json::Value::Null
    };
    drop(std::hint::black_box((scan, provisional)));
    (started.elapsed().as_nanos(), identity)
}

#[test]
fn af002_manifest_integrated_cost() {
    let _serial = super::super::super::super::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for (count, files, padding, overlap, missing) in [
        (1, 1, 0, false, false),
        (1, 32, 0, false, false),
        (4, 8, 0, false, false),
        (8, 4, 0, false, false),
        (1, 32, 4096, false, false),
        (4, 8, 4096, false, false),
        (4, 8, 0, true, false),
        (8, 4, 0, true, false),
        (4, 8, 4096, true, false),
        (4, 0, 0, false, false),
        (4, 8, 0, false, true),
        (4, 8, 0, true, true),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let mut roots = Vec::new();
        for i in 0..count {
            let root = if overlap {
                temp.path()
                    .join("shared")
                    .join((0..i).map(|_| "nested").collect::<PathBuf>())
            } else {
                temp.path().join(format!("r{i}"))
            };
            std::fs::create_dir_all(&root).unwrap();
            for j in 0..files {
                std::fs::write(
                    root.join(format!("f{i}_{j}.rs")),
                    format!("// {}\nstruct Wanted{i}_{j};\n", "共有".repeat(padding)),
                )
                .unwrap();
            }
            roots.push(root);
        }
        roots.push(roots[0].clone());
        if missing {
            roots.push(temp.path().join("missing"));
        }
        if missing && overlap {
            std::fs::write(roots[0].join("malformed.rs"), [0xff, 0xfe]).unwrap();
        }
        for proof in [false, true] {
            let (_, before) = run(&roots, proof, true, true);
            let (_, after) = run(&roots, proof, false, true);
            for key in ["candidates", "attempted", "complete", "provisional"] {
                assert_eq!(before[key], after[key], "Integrated {key} mismatch");
            }
            if proof {
                assert_eq!(before, after);
            } else {
                assert_eq!(after["work"]["root_manifests"], 0);
                assert_eq!(after["work"]["entry_clones"], 0);
                assert!(
                    !after["work"]["combined_receipts"]
                        .as_array()
                        .unwrap()
                        .is_empty()
                );
            }
            for _ in 0..3 {
                run(&roots, proof, true, false);
                run(&roots, proof, false, false);
            }
            let mut old = Vec::new();
            let mut new = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    old.push(run(&roots, proof, true, false).0);
                    new.push(run(&roots, proof, false, false).0);
                } else {
                    new.push(run(&roots, proof, false, false).0);
                    old.push(run(&roots, proof, true, false).0);
                }
            }
            old.sort_unstable();
            new.sort_unstable();
            eprintln!(
                "AF002_MANIFEST_INTEGRATED_COST {}",
                json!({"roots":count,
                "files_per_root":files,"padding":padding,"overlap":overlap,"missing":missing,
                "proof_required":proof,"original_median_ns":old[12],"integrated_median_ns":new[12],
                "original_samples_ns":old,"integrated_samples_ns":new,
                "original_observation":before,"integrated_observation":after})
            );
        }
    }
}

fn protected_path(caller: &str, scan: &str, fallback: &str, execution: &str) -> bool {
    let compact = |s: &str| s.split_whitespace().collect::<String>();
    let caller = compact(caller);
    let scan = compact(scan);
    let fallback = compact(fallback);
    let execution = compact(execution);
    caller.matches("authority.proof_target().is_some(),").count() == 2
        && caller.contains("filesystem::scan_with_manifest_demand(")
        && scan.contains("scan_with_manifest_demand(state,roots,query_name,true)")
        && scan.contains("ifmanifests_required{root_inventories(roots,&manifest)}else{Vec::new()}")
        && scan.contains("letcandidates=type_ref_candidate_owners(&manifest,query_name);")
        && fallback.contains("filesystem::scan_with_manifest_demand(state,&pending_roots,query_name,manifests_required,")
        && execution.contains("super::filesystem::scan(state,&roots,query_name)")
        && execution.contains("authority.proof_target().is_some()&&cached_satisfaction.is_none()")
}

#[test]
fn af002_manifest_integrated_static() {
    let caller = include_str!("../../mcp/tool_handlers/hydration.rs");
    let scan = include_str!("../../mcp/tool_handlers/hydration/filesystem.rs");
    let fallback = include_str!("../../mcp/tool_handlers/hydration/filesystem_discovery.rs");
    let execution = include_str!("../../mcp/tool_handlers/hydration/execution.rs");
    assert!(protected_path(caller, scan, fallback, execution));
    assert!(!protected_path(
        &caller.replace("authority.proof_target().is_some(),", "false,"),
        scan,
        fallback,
        execution
    ));
    assert!(!protected_path(
        caller,
        &scan.replace("if manifests_required", "if true"),
        fallback,
        execution
    ));
    assert!(!protected_path(
        caller,
        &scan.replace("query_name, true)", "query_name, false)"),
        fallback,
        execution
    ));
    assert!(!protected_path(
        caller,
        scan,
        fallback,
        &execution.replace("cached_satisfaction.is_none()", "false")
    ));
}
