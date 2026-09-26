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
