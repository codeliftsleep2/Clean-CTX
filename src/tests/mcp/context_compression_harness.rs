use std::fs;
use std::path::Path;

#[test]
fn focused_edit_economics_uses_a_real_focused_production_request() {
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("verification/context-compression/scripts/Capture-Baselines.ps1");
    let script = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", script_path.display()));

    assert!(
        script.contains("$responses[\"edit-focused\"] = Invoke-CleanCtxTool"),
        "the focused economics lane must own a distinct production response"
    );
    assert!(
        script.contains("focusMethods = @($focusTarget)"),
        "the focused production request must send the qualified focus target"
    );
    assert!(
        script.contains(
            "@{ fidelity = \"edit\";   focusCsv = $focusTarget; focusMode = \"focused\";    capture = \"edit-focused\" }"
        ),
        "the focused capture must not reuse the unfocused all-bodies Edit response"
    );
}

#[test]
fn focused_edit_economics_keeps_actual_content_separate_from_oracle_ir_source() {
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("verification/context-compression/scripts/Capture-Baselines.ps1");
    let script = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", script_path.display()));

    assert!(
        script.contains("oracle-ir-response.json"),
        "focused provide output has no canonical IR, so oracle rendering needs a distinct IR source"
    );
    assert!(
        script.contains("$responses[\"edit\"] | ConvertTo-Json -Depth 100"),
        "the focused oracle must derive from the complete Edit IR rather than replacing actual focused content"
    );
    assert!(
        script.contains("& $measure oracle $oracleRenderResponsePath"),
        "oracle rendering must use the selected IR-bearing response path"
    );
}

#[test]
fn measurement_helper_prefers_complete_named_ir_over_reduced_auxiliary_ir() {
    let helper_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("verification/context-compression/scripts/measure.rs");
    let helper = fs::read_to_string(&helper_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", helper_path.display()));

    let decoder_start = helper
        .find("fn complete_ir_from_response")
        .expect("the measurement helper must centralize complete-IR selection");
    let decoder = &helper[decoder_start..];
    let pretty_position = decoder
        .find("pointer(\"/result/pretty\")")
        .expect("complete named result.pretty must be an accepted IR source");
    let reduced_position = decoder
        .find("pointer(\"/result/ir\")")
        .expect("reduced result.ir must remain available as a compatibility fallback");

    assert!(
        pretty_position < reduced_position,
        "complete named IR must be preferred over reduced auxiliary IR"
    );
    assert!(
        decoder.contains("clean_ctx::ir::wire::wire_to_ir_detect"),
        "the complete named wire format must use the production format detector"
    );
    assert_eq!(
        helper
            .matches("complete_ir_from_response(&response)")
            .count(),
        2,
        "both production-candidate and oracle rendering must share the complete-IR boundary"
    );
}
