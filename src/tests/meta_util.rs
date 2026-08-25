// src/tests/meta_util.rs
//
// Tests for the canonical trilingual class-source span contract (C-22).
// Covers: TS decorators, C# attributes, Java annotations, non-decorated
// fallback, edge cases.

use crate::compression::capture_pipeline::CapEntry;

// ── find_class_source_start: TS decorators ────────────────────────

#[test]
fn ts_decorator_simple() {
    let source = "@Component()\nexport class Foo {}";
    // "class" starts at byte 13 (after "@Component()\n")
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(start, 0, "should find the '@' at byte 0");
}

#[test]
fn ts_decorator_with_args() {
    let source = "@NgModule({ imports: [] })\nexport class AppModule {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(start, 0, "should find the '@' before NgModule");
}

#[test]
fn ts_stacked_decorators() {
    let source = "@Directive({ selector: '[appHighlight]' })\n@Injectable()\nexport class HighlightDirective {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(start, 0, "should find the first '@' before Directive");
}

// ── find_class_source_start: C# attributes ────────────────────────

#[test]
fn csharp_attribute_simple() {
    let source = "[ApiController]\npublic class WeatherController {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(
        start,
        source.find('[').unwrap(),
        "should find the '[' at the attribute start"
    );
}

#[test]
fn csharp_stacked_attributes() {
    let source =
        "[ApiController]\n[Route(\"api/[controller]\")]\npublic class WeatherController {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(
        start,
        source.find('[').unwrap(),
        "should find the first '['"
    );
}

#[test]
fn csharp_attribute_with_modifier() {
    let source = "[Authorize]\npublic sealed class AdminController {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(
        start,
        source.find('[').unwrap(),
        "should find the '[' before modifiers"
    );
}

// ── find_class_source_start: Java annotations ─────────────────────

#[test]
fn java_annotation_simple() {
    let source = "@RestController\npublic class UserController {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(start, 0, "should find the '@' at byte 0");
}

#[test]
fn java_stacked_annotations() {
    let source = "@RestController\n@RequestMapping(\"/api/users\")\npublic class UserController {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(start, 0, "should find the first '@'");
}

#[test]
fn java_annotation_with_modifiers() {
    let source = "@Override\npublic final class ConcreteStrategy {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(start, source.find('@').unwrap(), "should find the '@'");
}

// ── find_class_source_start: non-decorated fallback ────────────────

#[test]
fn no_decorator_returns_class_pos() {
    let source = "export class Foo {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(
        start, class_pos,
        "non-decorated class returns class_pos unchanged"
    );
}

#[test]
fn no_decorator_with_modifiers() {
    let source = "public abstract class Foo {}";
    let class_pos = source.find("class ").unwrap();
    let start = crate::meta_util::find_class_source_start(source, class_pos);
    assert_eq!(
        start, class_pos,
        "modifier-only class returns class_pos unchanged"
    );
}

// ── class_source_from_capture: full contract ──────────────────────

fn make_cap(name: &str, raw_text: &str, start_byte: usize) -> CapEntry {
    CapEntry {
        name: name.to_string(),
        text: String::new(), // not used by class_source_from_capture
        raw_text: raw_text.to_string(),
        start_byte,
    }
}

#[test]
fn class_source_from_capture_ts_decorated() {
    let source = "@Component({ selector: 'app-x' })\nexport class X {}";
    let raw_text = "class X {}";
    let start_byte = source.find(raw_text).expect("raw_text must be in source");
    let cap = make_cap("class.root", raw_text, start_byte);
    let result = crate::meta_util::class_source_from_capture(source, &cap);
    assert!(
        result.contains("@Component"),
        "must include the decorator, got: {:?}",
        result
    );
    assert!(
        result.contains("class X {}"),
        "must include the class body, got: {:?}",
        result
    );
}

#[test]
fn class_source_from_capture_csharp_attributed() {
    let source = "[ApiController]\npublic class WeatherController {}";
    let raw_text = "class WeatherController {}";
    let start_byte = source.find(raw_text).expect("raw_text must be in source");
    let cap = make_cap("class.root", raw_text, start_byte);
    let result = crate::meta_util::class_source_from_capture(source, &cap);
    assert!(
        result.contains("[ApiController]"),
        "must include the attribute, got: {:?}",
        result
    );
    assert!(
        result.contains("class WeatherController {}"),
        "must include the class body, got: {:?}",
        result
    );
}

#[test]
fn class_source_from_capture_java_annotated() {
    let source = "@RestController\npublic class UserController {}";
    let raw_text = "class UserController {}";
    let start_byte = source.find(raw_text).expect("raw_text must be in source");
    let cap = make_cap("class.root", raw_text, start_byte);
    let result = crate::meta_util::class_source_from_capture(source, &cap);
    assert!(
        result.contains("@RestController"),
        "must include the annotation, got: {:?}",
        result
    );
    assert!(
        result.contains("class UserController {}"),
        "must include the class body, got: {:?}",
        result
    );
}

#[test]
fn class_source_from_capture_non_decorated() {
    let source = "export class Foo {}";
    let cap = make_cap("class.root", "class Foo {}", 7);
    let result = crate::meta_util::class_source_from_capture(source, &cap);
    assert_eq!(
        result, "class Foo {}",
        "non-decorated class returns raw_text as-is"
    );
}

#[test]
fn class_source_from_capture_empty_source() {
    let source = "";
    let cap = make_cap("class.root", "", 0);
    let result = crate::meta_util::class_source_from_capture(source, &cap);
    assert_eq!(result, "", "empty source returns empty slice");
}

// ── C-22 identity regression: class_source_from_capture round-trip ─

#[test]
fn class_source_from_capture_c22_identity() {
    // The capture text must be reconstructible from the source+CapEntry.
    // This is the C-22 invariant: class_source_from_capture must return
    // exactly the source text that the meta-layer extractors need.
    let source =
        "@Injectable({ providedIn: 'root' })\nexport class UserService {\n  constructor() {}\n}";
    let raw_text = "class UserService {\n  constructor() {}\n}";
    let start_byte = source.find(raw_text).unwrap();
    let cap = make_cap("class.root", raw_text, start_byte);
    let result = crate::meta_util::class_source_from_capture(source, &cap);

    // The result should start with the decorator and contain the class.
    let expected_start = source.find("@Injectable").unwrap();
    let expected_end = start_byte + raw_text.len();
    let expected = &source[expected_start..expected_end];
    assert_eq!(
        result, expected,
        "C-22 identity broken: class_source_from_capture must reconstruct the exact span"
    );
}
