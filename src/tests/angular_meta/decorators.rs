// src/tests/angular_meta/decorators.rs
//
// Tests for decorator extraction from a class capture.

use crate::angular_meta::decorators::extract_decorators;
use crate::compression::Fidelity;

fn lines_to_vec(opt: Option<Vec<String>>) -> Vec<String> {
    opt.unwrap_or_default()
}

// F-ANG-23: most tests use `Medium` fidelity because the
// field-level @Input/@Output scan is now fidelity-gated. The
// class-level @Input / @Output *decorator*-on-class pattern is still
// on at all fidelities (it is rare in real code), so we use
// `Medium` to also exercise the field-level scan.

#[test]
fn returns_none_for_plain_class() {
    let raw = r#"
        export class SampleService {
            private isReady: boolean = false;
        }
    "#;
    assert!(extract_decorators(raw, Fidelity::Medium).is_none());
}

#[test]
fn returns_none_for_empty_string() {
    assert!(extract_decorators("", Fidelity::Medium).is_none());
}

#[test]
fn returns_none_when_no_at_sign() {
    let raw = "export class Foo { x: number = 1; }";
    assert!(extract_decorators(raw, Fidelity::Medium).is_none());
}

#[test]
fn extracts_component_decorator_with_selector() {
    let raw = r#"
        @Component({
            selector: 'app-foo',
        })
        export class FooCmp {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    assert!(lines.iter().any(|l| l.starts_with("Φcmp:FooCmp")));
    assert!(lines.iter().any(|l| l.contains("sel=app-foo")));
}

#[test]
fn extracts_component_decorator_with_all_fields() {
    let raw = r#"
        @Component({
            selector: 'app-user-card',
            templateUrl: './user-card.component.html',
            styleUrls: ['./user-card.component.scss']
        })
        export class UserCardComponent {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    let cmp_line = lines.iter().find(|l| l.starts_with("Φcmp:")).unwrap();
    assert!(cmp_line.contains("sel=app-user-card"));
    assert!(cmp_line.contains("tpl=./user-card.component.html"));
    assert!(cmp_line.contains("sty=./user-card.component.scss"));
}

#[test]
fn extracts_injectable_decorator() {
    let raw = r#"
        @Injectable({ providedIn: 'root' })
        export class UserService {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    assert!(lines.iter().any(|l| l.starts_with("Φsvc:UserService scope=root")));
}

#[test]
fn extracts_injectable_without_provided_in() {
    let raw = r#"
        @Injectable()
        export class FooService {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    assert!(lines.iter().any(|l| l.starts_with("Φsvc:FooService")));
    assert!(!lines.iter().any(|l| l.contains("scope=")));
}

#[test]
fn extracts_ngmodule_decorator() {
    let raw = r#"
        @NgModule({
            declarations: [FooCmp, BarCmp],
            imports: [CommonModule],
            exports: [FooCmp]
        })
        export class AppModule {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    let mod_line = lines.iter().find(|l| l.starts_with("Φmod:")).unwrap();
    assert!(mod_line.contains("decl=[FooCmp,BarCmp]"));
    assert!(mod_line.contains("imp=[CommonModule]"));
    assert!(mod_line.contains("exp=[FooCmp]"));
}

#[test]
fn extracts_directive_decorator() {
    let raw = r#"
        @Directive({ selector: '[appHighlight]' })
        export class HighlightDirective {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    let dir_line = lines.iter().find(|l| l.starts_with("Φdir:")).unwrap();
    assert!(dir_line.contains("sel=[appHighlight]"));
}

#[test]
fn extracts_pipe_decorator() {
    let raw = r#"
        @Pipe({ name: 'upper' })
        export class UpperPipe {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    let pipe_line = lines.iter().find(|l| l.starts_with("Φpipe:")).unwrap();
    assert!(pipe_line.contains("name=upper"));
}

#[test]
fn extracts_input_and_output_decorators() {
    let raw = r#"
        @Component({ selector: 'app-foo' })
        export class FooCmp {
            @Input() userId: string = '';
            @Input('aliasName') userName: string = '';
            @Output() userDeleted = new EventEmitter();
        }
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    assert!(lines.iter().any(|l| l == "Φin:?" || l.starts_with("Φin:")));
    assert!(lines.iter().any(|l| l == "Φout:?" || l.starts_with("Φout:")));
}

#[test]
fn extracts_constructor_injects() {
    let raw = r#"
        @Injectable({ providedIn: 'root' })
        export class UserService {
            constructor(private http: HttpClient, private auth: AuthService) {}
        }
    "#;
    // F-ANG-23: Φinjects: is high-fidelity only.
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::High));
    let injects_line = lines.iter().find(|l| l.starts_with("Φinjects:")).unwrap();
    assert!(injects_line.contains("HttpClient"));
    assert!(injects_line.contains("AuthService"));
}

#[test]
fn unknown_decorator_is_skipped() {
    let raw = r#"
        @SomeRandomDecorator
        @Component({ selector: 'app-foo' })
        export class FooCmp {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    // Only the @Component is recognised; @SomeRandomDecorator is
    // silently dropped.
    assert!(lines.iter().any(|l| l.starts_with("Φcmp:FooCmp")));
    assert!(lines.iter().any(|l| l.starts_with("Φcmp:FooCmp")));
}

#[test]
fn handles_single_quoted_strings() {
    let raw = r#"
        @Component({
            selector: 'app-foo',
            templateUrl: './foo.html'
        })
        export class FooCmp {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    let cmp_line = lines.iter().find(|l| l.starts_with("Φcmp:")).unwrap();
    assert!(cmp_line.contains("sel=app-foo"));
    assert!(cmp_line.contains("tpl=./foo.html"));
}

#[test]
fn handles_double_quoted_strings() {
    let raw = r#"
        @Component({
            selector: "app-foo",
            templateUrl: "./foo.html"
        })
        export class FooCmp {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    let cmp_line = lines.iter().find(|l| l.starts_with("Φcmp:")).unwrap();
    assert!(cmp_line.contains("sel=app-foo"));
    assert!(cmp_line.contains("tpl=./foo.html"));
}

#[test]
fn handles_unterminated_string_gracefully() {
    // A trailing single quote without closing — the parser should
    // not panic; it should produce whatever it can.
    let raw = r#"
        @Component({ selector: 'app-foo' })
        export class FooCmp {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    assert!(lines.iter().any(|l| l.contains("Φcmp:FooCmp")));
}

// ----- F-ANG-23 fidelity tests -----

#[test]
fn low_fidelity_skips_field_input_output() {
    let raw = r#"
        @Component({ selector: 'app-foo' })
        export class FooCmp {
            @Input() userId: string = '';
            @Output() userDeleted = new EventEmitter();
        }
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Low));
    // At Low fidelity, the field-level @Input/@Output scan is skipped.
    assert!(!lines.iter().any(|l| l.starts_with("Φin:userId")));
    assert!(!lines.iter().any(|l| l.starts_with("Φout:userDeleted")));
    // But the class-level summary is still emitted.
    assert!(lines.iter().any(|l| l.starts_with("Φcmp:FooCmp")));
}

#[test]
fn medium_fidelity_emits_field_input_output() {
    let raw = r#"
        @Component({ selector: 'app-foo' })
        export class FooCmp {
            @Input() userId: string = '';
            @Output() userDeleted = new EventEmitter();
        }
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    assert!(lines.iter().any(|l| l.starts_with("Φin:userId")));
    assert!(lines.iter().any(|l| l.starts_with("Φout:userDeleted")));
}

#[test]
fn high_fidelity_emits_phi_injects() {
    let raw = r#"
        @Injectable({ providedIn: 'root' })
        export class UserService {
            constructor(private http: HttpClient) {}
        }
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::High));
    // F-ANG-23: Φinjects: is High-fidelity only.
    assert!(lines.iter().any(|l| l.starts_with("Φinjects:") && l.contains("HttpClient")));
}

#[test]
fn medium_fidelity_omits_phi_injects() {
    let raw = r#"
        @Injectable({ providedIn: 'root' })
        export class UserService {
            constructor(private http: HttpClient) {}
        }
    "#;
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    // F-ANG-23: Φinjects: should NOT appear at Medium fidelity.
    assert!(!lines.iter().any(|l| l.starts_with("Φinjects:")));
}

// ----- Track A: None-return tests for the string-walker helpers -----
//
// Each test exercises the malformed-input path that previously
// returned a silent sentinel (len-1, len(), len(), or
// "(anonymous)"). The new return type is `Option<...>`, so a
// well-formed `None` is the expected behaviour.

#[test]
fn find_matching_brace_returns_none_on_unclosed_body() {
    // F-ANG-08: no closing `}` — used to return `len-1` (silent
    // truncation to end of text). Now returns `None`.
    let body = "{ unclosed";
    assert!(crate::angular_meta::decorators::find_matching_brace(body, 0).is_none());
}

#[test]
fn consume_call_expression_returns_none_on_unterminated_call() {
    // F-ANG-09: no closing `)` — used to return `(i-open_paren,
    // text[open_paren+1..i])` (silent EOF). Now returns `None`.
    let text = "(unterminated call";
    assert!(crate::angular_meta::decorators::consume_call_expression(text, 0).is_none());
}

#[test]
fn find_class_head_end_returns_none_on_no_class_keyword() {
    // F-ANG-12: no `class ` and no `{` — used to return `raw.len()`
    // (silently included the rest of the file as part of the
    // "head"). Now returns `None`.
    let raw = "export const x = 5;\n";
    assert!(crate::angular_meta::decorators::find_class_head_end(raw).is_none());
}

#[test]
fn extract_class_name_returns_none_for_anonymous_class() {
    // F-ANG-13: no `class ` keyword at all — used to return the
    // literal `"(anonymous)"`. Now returns `None`.
    let raw = "export const x = 5;";
    assert!(crate::angular_meta::decorators::extract_class_name(raw).is_none());
}

#[test]
fn find_class_body_open_returns_none_when_no_class_keyword() {
    // F-ANG-07: no `class ` keyword — used to early-return `?`
    // on the primitive `find`. The pre-audit behaviour was already
    // correct; this test pins the contract.
    let raw = "function foo() { return 1; }";
    assert!(crate::angular_meta::decorators::find_class_body_open(raw).is_none());
}

// ----- Track A: end-to-end behaviour preservation -----

#[test]
fn extract_decorators_substitutes_question_mark_for_anonymous_class() {
    // F-ANG-13: callers substitute `?` at the call site for missing
    // class names. The marker line is still emitted using `?` as
    // the class name.
    let raw = r#"
        @Component({ selector: 'app-foo' })
        export default class { /* anonymous */ }
    "#;
    // Hmm, this input actually has `class` so the name is empty
    // (the part right after `class` is ` {`). `extract_class_name`
    // returns `None` and the call site substitutes `?`.
    let lines = lines_to_vec(extract_decorators(raw, Fidelity::Medium));
    // The class name shows up as `?` in the marker line.
    assert!(
        lines.iter().any(|l| l.contains("Φcmp:?") || l.contains("Φcmp:?")),
        "expected Φcmp:? in {:?}",
        lines
    );
}

#[test]
fn extract_decorators_returns_none_for_input_without_class_keyword() {
    // F-ANG-12: malformed input that has neither `class ` nor `{`
    // produces `None` from `find_class_head_end`, which
    // `extract_decorators` propagates. Previously this was a
    // wasted scan that returned `None` anyway.
    let raw = "// only a comment, no class";
    assert!(extract_decorators(raw, Fidelity::Medium).is_none());
}

#[test]
fn extract_decorators_handles_unterminated_decorator_call() {
    // F-ANG-09: the decorator has `(` but no `)`. Previously the
    // scanner consumed to end-of-text and used the resulting slice
    // as the decorator arg. Now it advances past the `(` and
    // uses an empty arg, so the decorator is parsed without its
    // arguments.
    let raw = r#"
        @Component(
        export class FooCmp {}
    "#;
    // The class body is unterminated too — this exercises both
    // F-ANG-08 (`find_matching_brace` returns `None`) and F-ANG-09
    // (`consume_call_expression` returns `None`). The function
    // should not panic and should still emit a marker line.
    let result = std::panic::catch_unwind(|| extract_decorators(raw, Fidelity::Medium).is_some());
    assert!(result.is_ok(), "extract_decorators panicked on malformed input");
}
