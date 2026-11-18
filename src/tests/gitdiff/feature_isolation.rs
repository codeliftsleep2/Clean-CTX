//! HTML dispatch must respect Angular availability without enabling its grammar.
use super::*;

#[test]
fn ci_runtime_feature_template_dispatch_matches_availability() {
    let before = "<div></div>\n";
    let after = "<div>{{name}}</div>\n<span></span>\n";
    let (body, error) = diff_two_contents(before, after, "sample.component.html", Fidelity::High);
    assert!(error.is_none(), "{error:?}");
    if cfg!(feature = "angular") {
        assert!(body.contains("template (old)"), "{body}");
        assert!(body.contains("template (new)"), "{body}");
    } else {
        assert_eq!(body, "  +1 lines (1 → 2)");
        assert_eq!(
            compress_added_file(after, "sample.component.html", Fidelity::High),
            format_line_delta(0, 2)
        );
    }
}
