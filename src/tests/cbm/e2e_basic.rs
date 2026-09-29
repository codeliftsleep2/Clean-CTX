use super::*;

#[test]
fn e2e_intelligence_layer_full_pipeline() {
    use crate::cbm::SymbolImportance;
    use crate::intelligence::compute_pagerank;
    use crate::intelligence::fidelity::{
        FidelityRecommendation, apply_recommendation, cbm_informed_fidelity,
    };
    use std::collections::HashMap;

    // Build IR scores
    let mut ir_scores = HashMap::new();
    ir_scores.insert("critical_handler".to_string(), 50.0);
    ir_scores.insert("helper_func".to_string(), 5.0);
    ir_scores.insert("unused_util".to_string(), 1.0);

    // Build CBM importance scores
    let mut cbm_scores = HashMap::new();
    cbm_scores.insert(
        "critical_handler".to_string(),
        SymbolImportance {
            symbol: "critical_handler".into(),
            score: 0.95,
            file: "src/handler.rs".into(),
        },
    );
    cbm_scores.insert(
        "helper_func".to_string(),
        SymbolImportance {
            symbol: "helper_func".into(),
            score: 0.5,
            file: "src/handler.rs".into(),
        },
    );
    cbm_scores.insert(
        "unused_util".to_string(),
        SymbolImportance {
            symbol: "unused_util".into(),
            score: 0.1,
            file: "src/utils.rs".into(),
        },
    );

    // Step 1: Compute PageRank
    let scores = compute_pagerank(ir_scores, cbm_scores, Some(0.6));
    assert!(!scores.is_empty(), "Should have combined scores");

    // Step 2: Verify high-importance symbol gets ForceHigh
    let critical_score = scores.get("critical_handler").unwrap();
    assert!(
        critical_score.combined_score > 0.7,
        "Critical handler should have high combined score: {}",
        critical_score.combined_score
    );

    // Step 3: Test fidelity recommendation pipeline
    let mut importance_map = HashMap::new();
    importance_map.insert(
        "critical_handler".to_string(),
        SymbolImportance {
            symbol: "critical_handler".into(),
            score: critical_score.combined_score,
            file: "src/handler.rs".into(),
        },
    );

    let rec = cbm_informed_fidelity(
        "src/handler.rs",
        &importance_map,
        FidelityRecommendation::NoRecommendation,
    );

    // Step 4: Apply recommendation
    let fidelity = apply_recommendation(&rec);
    if critical_score.combined_score > 0.8 {
        assert_eq!(fidelity, Some(crate::compression::Fidelity::High));
    }
}

// ---- K-1: Indexing lifecycle tests ---------------------------------

/// Prove `ensure_indexed()` is report-only -- it does NOT transition
/// `NotStarted` -> `InProgress` (no spawn, no mutation).
/// Uses a mock bridge with `Available` status but `NotStarted` state.
#[test]
fn ensure_indexed_does_not_trigger_indexing() {
    use crate::cbm::bridge::test_helpers::new_available_not_started;

    let mut bridge = new_available_not_started();

    // Precondition: state is NotStarted (empty map).
    {
        let states = bridge.indexing_state();
        assert!(
            states.is_empty(),
            "precondition: indexing state should be empty (NotStarted)"
        );
    }

    // Call ensure_indexed -- in the OLD code this would spawn a background
    // thread, flip state to InProgress, and return StillIndexing.
    // In the NEW code it must NOT mutate state and return StillIndexing.
    let result = bridge.ensure_indexed();

    // Must return Ok(StillIndexing) -- not Err, not Ready.
    assert!(
        result.is_ok(),
        "ensure_indexed should return Ok(StillIndexing) when Available/NotStarted: {:?}",
        result
    );
    match result.unwrap() {
        crate::cbm::bridge::IndexingStatus::StillIndexing { elapsed_secs } => {
            assert_eq!(elapsed_secs, 0, "should report 0 elapsed");
        }
        _ => panic!("expected StillIndexing, got something else"),
    }

    // State must remain NotStarted (no transition to InProgress) -- proving no spawn occurred.
    let states = bridge.indexing_state();
    for (project, state) in states.iter() {
        assert!(
            matches!(state, crate::cbm::bridge::IndexingState::NotStarted),
            "K-1: ensure_indexed must NOT mutate indexing state to InProgress (no spawn) -- project '{project}' is {:?}",
            state
        );
    }
}

/// Prove indexing begins at bridge construction and reaches a non-`NotStarted`
/// state.
///
/// Uses a small dedicated root because this assertion is about construction-
/// time state publication, not successful indexing of the repository-wide
/// shared fixture. A terminal CBM indexing failure is still evidence that the
/// constructor started the indexer; the shared live scenarios separately own
/// successful-index requirements.
#[serial(cbm_live)]
#[test]
fn try_create_begins_indexing_at_construction() {
    use crate::cbm::bridge::IndexingState;

    let cbm_available = cbm_binary_exists();
    if !cbm_available {
        eprintln!("Skipping -- CBM not installed");
        return;
    }
    let fixture = tempfile::tempdir().expect("construction-index fixture");
    std::fs::write(
        fixture.path().join("construction.ts"),
        "export class ConstructionProbe { run(): number { return 1; } }\n",
    )
    .expect("construction-index source");
    let config = crate::cbm::config::CbmConfig {
        enabled: true,
        ..Default::default()
    };
    let bridge = crate::cbm::bridge::GraphBridge::try_create(&config, fixture.path());
    assert!(bridge.is_available(), "live CBM bridge must launch");

    // Valid lifecycle: the bridge started indexing at construction.
    // The `indexing_state` map must NOT be empty (an empty map means
    // `NotStarted` -- the construction-time spawn never ran), and NO
    // project may be `NotStarted` (a `NotStarted` entry can only appear if a
    // query switched to an un-indexed project via `set_project` and
    // `ensure_indexed()` lazily inserted it -- the K-1 regression this
    // test guards against).
    //
    // Snapshot an owned copy while holding the indexing-state guard.
    let states = {
        bridge
            .indexing_state()
            .iter()
            .map(|(project, state)| (project.clone(), state.clone()))
            .collect::<std::collections::HashMap<String, IndexingState>>()
    };
    assert!(
        !states.is_empty(),
        "K-1: indexing_state is empty after bridge construction -- \
         the construction-time async indexer never ran."
    );
    for (project, state) in &states {
        match state {
            IndexingState::InProgress { .. } | IndexingState::Complete => {
                // Good: indexing was kicked off at construction.
            }
            IndexingState::Failed(msg) => {
                // Acceptable: CBM binary may not be compatible.
                eprintln!("Note: indexing started but failed: {msg}");
            }
            IndexingState::NotStarted => {
                panic!(
                    "K-1: project '{project}' is NotStarted -- \
                     this means the background indexer was never spawned at construction."
                );
            }
        }
    }
}
