//! Actual public compression -> compilation alias transform; no synthetic caller bypass.
use crate::work_observations_af_perf_002 as work;
use serde_json::json;
#[test]
fn af002_c2_public_alias_and_type_operand_axes() {
    let _serial = crate::protocol::handler_response_serial();
    let mut dimensions: Vec<_> = [0, 1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 4))
        .collect();
    dimensions.extend([1, 2, 4, 8, 16, 64, 256].into_iter().map(|n| (16, n)));
    for (aliases, parameters) in dimensions {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("types.ts");
        let parameter_source = (0..parameters)
            .map(|i| format!("p{i}: DomainType0000"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "export class Workload {{ run({parameter_source}): DomainType0000 {{ return p0; }} }}"
        );
        std::fs::write(&file, &source).unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.type_aliases = (0..aliases)
            .map(|i| (format!("DomainType{i:04}"), format!("$t{i}")))
            .collect();
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        work::reset();
        crate::protocol::captured_responses().clear();
        let start = std::time::Instant::now();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            "compress_code_context",
            &json!({"arguments":{
                "filePath":file,"workspaceRoot":root.path(),"fidelity":"high"
            }}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let ops = response["result"]["pretty"]["ir"]
            .as_array()
            .expect("named IR response");
        let declared: Vec<_> = ops.iter().filter(|op| op[0] == "TYPE").cloned().collect();
        assert_eq!(
            declared,
            if aliases == 0 {
                vec![]
            } else {
                vec![json!(["TYPE", "$t0", "DomainType0000"])]
            }
        );
        let expected_type = if aliases == 0 {
            "DomainType0000"
        } else {
            "$t0"
        };
        let signatures: Vec<_> = ops.iter().filter(|op| op[0] == "SIG").collect();
        assert_eq!(signatures.len(), parameters, "captured parameter growth");
        for (i, signature) in signatures.iter().enumerate() {
            assert_eq!(signature[3], expected_type);
            assert_eq!(signature[4], format!("p{i}"));
        }
        let returns: Vec<_> = ops.iter().filter(|op| op[0] == "RET").collect();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0][2], expected_type);
        let c = work::snapshot();
        assert!(
            c.alias_ir_calls > 0,
            "production post-compilation alias pass reached"
        );
        if aliases > 0 {
            assert!(
                c.alias_ir_operands > parameters,
                "actual captured operands must represent parameter and return types"
            );
        }
        eprintln!(
            "AF002_C2_PUBLIC aliases={aliases} parameters={parameters} source_bytes={} ir_calls={} instruction_visits={} operands={} matcher_calls={} input_bytes={} setup_entries={} eligible_entries={} sort_keys={} positions={} probes={} matches={} pair_capacity_sum={} us={} boundary=whole-public-compression",
            source.len(),
            c.alias_ir_calls,
            c.alias_ir_instruction_visits,
            c.alias_ir_operands,
            c.alias_calls,
            c.alias_input_bytes,
            c.alias_setup_entries,
            c.alias_eligible_entries,
            c.alias_sort_key_calls,
            c.alias_positions,
            c.alias_probes,
            c.alias_matches,
            c.alias_pair_capacity,
            start.elapsed().as_micros()
        );
    }
}
