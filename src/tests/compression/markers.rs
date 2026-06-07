use super::*;

#[test]
fn build_marker_known_captures() {
    assert_eq!(build_marker("throw.root", "new Error()"), Some("⊕!new Error()".to_string()));
    assert_eq!(build_marker("for.root", ""), Some("⊕loop".to_string()));
    assert_eq!(build_marker("if.root", ""), Some("⊕guard".to_string()));
    assert_eq!(build_marker("while.root", ""), Some("⊕loop".to_string()));
    assert_eq!(build_marker("return.root", "result"), Some("⊕⇒result".to_string()));
}

#[test]
fn build_marker_unknown_returns_none() {
    assert_eq!(build_marker("class.root", "Foo"), None);
    assert_eq!(build_marker("method.root", "foo()"), None);
    assert_eq!(build_marker("field.root", "x:number"), None);
    assert_eq!(build_marker("import.root", "import x"), None);
    assert_eq!(build_marker("", ""), None);
}

#[test]
fn expand_markers_in_line_works() {
    let input = "if(x)⊕guard foo()⊕loop bar()⊕⇒result ⊕!Error";
    let expanded = expand_markers_in_line(input);
    assert_eq!(expanded, "if(x) foo() bar()→ result throws: Error");
}

#[test]
fn expand_marker_known_tokens() {
    assert_eq!(expand_marker("⊕guard"), Some(""));
    assert_eq!(expand_marker("⊕loop"), Some(""));
    assert_eq!(expand_marker("⊕⇒"), Some("→ "));
    assert_eq!(expand_marker("⊕!"), Some("throws: "));
}

#[test]
fn expand_marker_unknown_returns_none() {
    assert_eq!(expand_marker("⊕nope"), None);
    assert_eq!(expand_marker("regular text"), None);
}