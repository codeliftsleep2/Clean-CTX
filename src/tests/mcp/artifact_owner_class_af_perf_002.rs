//! A2 owner/class axes; independent of A3 selection implementation.
use super::{assert_published_exact_call, assert_save_call, query};
use crate::work_observations_af_perf_002 as work;

#[test]
fn af002_a2_owner_class_public_growth() {
    let _serial = crate::protocol::handler_response_serial();
    // Hold class count at 32 for the owner axis, then hold owners at two.
    for (owners, classes) in [
        (2, 32),
        (4, 32),
        (8, 32),
        (16, 32),
        (2, 2),
        (2, 4),
        (2, 8),
        (2, 16),
        (2, 64),
        (2, 256),
    ] {
        let root = tempfile::tempdir().unwrap();
        for owner in 0..owners {
            let (name, mut source) = match owner {
                0 => (
                    "Caller.cs".to_owned(),
                    "class Caller { void Run(Target target) { target.Save(); } }".to_owned(),
                ),
                1 => (
                    "Target.cs".to_owned(),
                    "class Target { public void Save() {} }".to_owned(),
                ),
                _ => (
                    format!("Owner{owner}.cs"),
                    format!("class Owner{owner} {{ Target target; }}"),
                ),
            };
            for extra in (owner..classes - owners).step_by(owners) {
                source.push_str(&format!(" class Extra{extra} {{}}"));
            }
            std::fs::write(root.path().join(name), source).unwrap();
        }
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        query(&state, root.path(), "TypeRef", "Target");
        let admitted = state
            .workspace_index_read()
            .file_map()
            .keys()
            .filter(|f| f.ends_with(".cs"))
            .count();
        assert_eq!(
            admitted, owners,
            "A2 acquisition prerequisite: every generated owner must be admitted"
        );
        state.mark_csharp_authority_dirty();
        state.reset_test_csharp_reconciliation_count();
        state.reset_test_full_semantic_parses();
        work::reset();
        let started = std::time::Instant::now();
        let first = query(&state, root.path(), "Method", "Save");
        assert_save_call(&first);
        assert_published_exact_call(&state, root.path());
        let dirty = work::snapshot();
        assert_eq!(
            dirty.artifact_projection_owners, owners,
            "actual projected generation population"
        );
        assert!(
            dirty.csharp_role_entries[5] > 0,
            "execution-context consumer must be reached"
        );
        let observed_classes = dirty.csharp_role_class_visits[5] / dirty.csharp_role_entries[5];
        assert_eq!(observed_classes, classes, "actual resolved class dimension");
        assert_eq!(
            dirty.csharp_role_class_visits[5],
            dirty.csharp_role_entries[5] * classes,
            "no truncated class-count attribution"
        );
        eprintln!(
            "AF002_A2 owners={owners} classes={classes} phase=dirty projection_owners={} role_entries={:?} class_visits={:?} ir_instruction_clones={} publication_accessor={:?} resolutions={} selection_visits={:?} reconciliations={} full_parses={} us={}",
            dirty.artifact_projection_owners,
            dirty.csharp_role_entries,
            dirty.csharp_role_class_visits,
            dirty.artifact_ir_instruction_clones,
            dirty.artifact_publication_clones,
            dirty.csharp_resolution_calls,
            dirty.csharp_declaration_probes,
            state.test_csharp_reconciliation_count(),
            state.test_full_semantic_parses(),
            started.elapsed().as_micros()
        );
        eprintln!(
            "AF002_A2_JOINS owners={owners} classes={classes} files={:?} method_owners={} raw_parts={:?} context_methods={} action_methods={}",
            dirty.csharp_class_file_visits,
            dirty.csharp_method_owner_visits,
            dirty.csharp_raw_part_visits,
            dirty.csharp_context_method_visits,
            dirty.csharp_aspnet_method_visits
        );
        work::reset();
        state.reset_test_csharp_reconciliation_count();
        state.reset_test_full_semantic_parses();
        let started = std::time::Instant::now();
        let repeated = query(&state, root.path(), "Method", "Save");
        assert_save_call(&repeated);
        assert_published_exact_call(&state, root.path());
        let warm = work::snapshot();
        eprintln!(
            "AF002_A2 owners={owners} classes={classes} phase=warm projection_owners={} role_entries={:?} class_visits={:?} reconciliations={} full_parses={} us={}",
            warm.artifact_projection_owners,
            warm.csharp_role_entries,
            warm.csharp_role_class_visits,
            state.test_csharp_reconciliation_count(),
            state.test_full_semantic_parses(),
            started.elapsed().as_micros()
        );
    }
}
