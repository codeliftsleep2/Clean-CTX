// src/tests/angular_meta/style.rs
//
// Tests for the CSS/SCSS style extractor.

use crate::angular_meta::style::{StyleShape, extract_style_shape};

#[test]
fn extracts_class_selectors() {
    let css = r#"
.card { border: 1px solid #ccc; }
.btn-primary { background: blue; }
"#;
    let shape = extract_style_shape(css);
    assert!(shape.class_selectors.contains(&".card".to_string()));
    assert!(shape.class_selectors.contains(&".btn-primary".to_string()));
}

#[test]
fn extracts_scss_variables() {
    let css = r#"
$primary-color: #1976d2;
$card-padding: 16px;
"#;
    let shape = extract_style_shape(css);
    assert!(shape.variables.contains(&"$primary-color".to_string()));
    assert!(shape.variables.contains(&"$card-padding".to_string()));
}

#[test]
fn extracts_css_custom_properties() {
    let css = r#"
:root {
  --bg-color: white;
  --text-color: black;
}
"#;
    let shape = extract_style_shape(css);
    assert!(shape.variables.contains(&"--bg-color".to_string()));
    assert!(shape.variables.contains(&"--text-color".to_string()));
}

#[test]
fn extracts_at_rules() {
    let css = r#"
.btn {
  @include flex-center;
  @mixin helper() {}
}
"#;
    let shape = extract_style_shape(css);
    assert!(shape.at_rules.contains(&"@include".to_string()));
    assert!(shape.at_rules.contains(&"@mixin".to_string()));
}

#[test]
fn skips_at_media_and_keyframes() {
    let css = r#"
@media (max-width: 768px) { .card { width: 100%; } }
@keyframes fade { from { opacity: 0; } }
"#;
    let shape = extract_style_shape(css);
    assert!(!shape.at_rules.contains(&"@media".to_string()));
    assert!(!shape.at_rules.contains(&"@keyframes".to_string()));
}

#[test]
fn skips_single_line_comments() {
    let css = r#"
// This is a comment with .fake-class
.card { color: red; }
"#;
    let shape = extract_style_shape(css);
    assert!(!shape.class_selectors.contains(&".fake-class".to_string()));
    assert!(shape.class_selectors.contains(&".card".to_string()));
}

#[test]
fn skips_multi_line_comments() {
    let css = r#"
/* Comment with .hidden-class */
.card { color: red; }
"#;
    let shape = extract_style_shape(css);
    assert!(!shape.class_selectors.contains(&".hidden-class".to_string()));
    assert!(shape.class_selectors.contains(&".card".to_string()));
}

#[test]
fn skips_quoted_strings() {
    let css = r#"
.card { content: ".fake"; }
"#;
    let shape = extract_style_shape(css);
    assert!(!shape.class_selectors.contains(&".fake".to_string()));
    assert!(shape.class_selectors.contains(&".card".to_string()));
}

#[test]
fn empty_css_returns_empty_shape() {
    let shape = extract_style_shape("");
    assert!(shape.class_selectors.is_empty());
    assert!(shape.variables.is_empty());
    assert!(shape.at_rules.is_empty());
}

#[test]
fn whitespace_only_returns_empty_shape() {
    let shape = extract_style_shape("   \n  \t  ");
    assert!(shape.class_selectors.is_empty());
}

#[test]
fn to_marker_line_contains_selectors() {
    let css = ".card { color: red; } .btn { color: blue; }";
    let shape = extract_style_shape(css);
    let line = shape.to_marker_line();
    assert!(line.starts_with("Φsty:"));
    assert!(line.contains(".card"));
}

#[test]
fn to_marker_line_contains_variables() {
    let css = "$primary: #1976d2;";
    let shape = extract_style_shape(css);
    let line = shape.to_marker_line();
    assert!(line.contains("$primary"));
}

#[test]
fn to_marker_line_contains_at_rules() {
    let css = ".btn { @include flex; }";
    let shape = extract_style_shape(css);
    let line = shape.to_marker_line();
    assert!(line.contains("@include"));
}

#[test]
fn to_marker_line_empty_when_no_content() {
    let shape = StyleShape::default();
    assert_eq!(shape.to_marker_line(), "Φsty:empty");
}

#[test]
fn deduplicates_selectors() {
    let css = ".card { color: red; } .card { color: blue; }";
    let shape = extract_style_shape(css);
    let count = shape
        .class_selectors
        .iter()
        .filter(|s| *s == ".card")
        .count();
    assert_eq!(count, 1, "class selectors should be deduplicated");
}

#[test]
fn deduplicates_at_rules() {
    let css = ".a { @include x; } .b { @include y; }";
    let shape = extract_style_shape(css);
    let count = shape.at_rules.iter().filter(|r| *r == "@include").count();
    assert_eq!(count, 1, "at-rules should be deduplicated");
}
