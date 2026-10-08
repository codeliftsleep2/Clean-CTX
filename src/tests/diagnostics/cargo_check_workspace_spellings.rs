use super::{TransformationFacts, workspace_text};

#[test]
fn opaque_workspace_path_regression_windows_native_plain_and_slash_spellings() {
    let root = r"\\?\C:\Users\Operator\work";
    let text = concat!(
        r"Compiling (C:\Users\Operator\work); ",
        r"failed at \\?\C:\Users\Operator\work\src\lib.rs; ",
        "failed at c:/users/operator/work/src/lib.rs"
    );
    let mut facts = TransformationFacts::default();
    let mapped = workspace_text(text, root, &mut facts, true);
    assert_eq!(
        mapped,
        concat!(
            "Compiling (<workspace>); ",
            r"failed at <workspace>\src\lib.rs; ",
            "failed at <workspace>/src/lib.rs"
        )
    );
    assert_eq!(facts.workspace_paths_mapped, 3);
}

#[test]
fn opaque_workspace_path_regression_windows_unc_spellings() {
    let root = r"\\?\UNC\server\share\work";
    let text = concat!(
        r"location \\server\share\work\src\lib.rs; ",
        "location //server/share/work/src/lib.rs"
    );
    let mut facts = TransformationFacts::default();
    let mapped = workspace_text(text, root, &mut facts, true);
    assert_eq!(
        mapped,
        concat!(
            r"location <workspace>\src\lib.rs; ",
            "location <workspace>/src/lib.rs"
        )
    );
    assert_eq!(facts.workspace_paths_mapped, 2);
}

#[test]
fn opaque_workspace_path_regression_text_mapping_does_not_claim_sibling_prefixes() {
    let root = "/tmp/\u{e9}-work";
    let text = "location /tmp/\u{e9}-work/src/lib.rs; unrelated /tmp/\u{e9}-work-other/src/lib.rs";
    let mut facts = TransformationFacts::default();
    let mapped = workspace_text(text, root, &mut facts, false);
    assert_eq!(
        mapped,
        "location <workspace>/src/lib.rs; unrelated /tmp/\u{e9}-work-other/src/lib.rs"
    );
    assert_eq!(facts.workspace_paths_mapped, 1);
}
