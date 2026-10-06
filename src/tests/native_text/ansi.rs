use super::*;

#[test]
fn removes_only_supported_csi_and_reports_facts() {
    let input = "\x1b[31mred\x1b[0m \x1b[?25hkeep \x1b]8;;https://x\x07link";
    let result = normalize_terminal_text(input);
    assert_eq!(result.text, "red \x1b[?25hkeep \x1b]8;;https://x\x07link");
    assert_eq!(result.facts.sequences_removed, 2);
    assert_eq!(result.facts.bytes_removed, 9);
}

#[test]
fn normalization_is_idempotent() {
    let once = normalize_terminal_text("\x1b[1;32mok\x1b[0m");
    let twice = normalize_terminal_text(&once.text);
    assert_eq!(once.text, twice.text);
    assert_eq!(twice.facts.sequences_removed, 0);
}
