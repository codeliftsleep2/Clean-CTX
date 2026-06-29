// src/tests/angular_meta/detect.rs
//
// Tests for the Angular detection heuristic.

use crate::angular_meta::detect::is_angular_file;

#[test]
fn detects_component_decorator() {
    let src = r#"@Component({selector: 'app-foo'}) class Foo {}"#;
    assert!(is_angular_file(src));
}

#[test]
fn detects_injectable_decorator() {
    let src = r#"@Injectable({providedIn: 'root'}) class FooSvc {}"#;
    assert!(is_angular_file(src));
}

#[test]
fn detects_ngmodule_decorator() {
    let src = r#"@NgModule({declarations: []}) class AppModule {}"#;
    assert!(is_angular_file(src));
}

#[test]
fn detects_directive_decorator() {
    let src = r#"@Directive({selector: '[appHighlight]'}) class HighlightDirective {}"#;
    assert!(is_angular_file(src));
}

#[test]
fn detects_pipe_decorator() {
    let src = r#"@Pipe({name: 'upper'}) class UpperPipe {}"#;
    assert!(is_angular_file(src));
}

#[test]
fn detects_viewchild_decorator() {
    let src = r#"class Foo { @ViewChild('bar') bar: any; }"#;
    assert!(is_angular_file(src));
}

#[test]
fn detects_weak_signal_with_angular_core_import() {
    let src = r#"
        import { Component, Input } from '@angular/core';
        class Foo { @Input() bar: string = ''; }
    "#;
    assert!(is_angular_file(src));
}

#[test]
fn rejects_plain_typescript() {
    let src = r#"
        export class SampleService {
            private isReady: boolean = false;
        }
    "#;
    assert!(!is_angular_file(src));
}

#[test]
fn rejects_comment_with_decorator_name() {
    // A-11: Comments should not trigger Angular detection
    let src = r#"
        // TODO: remove @Component when refactoring
        export class SampleService {
            private isReady: boolean = false;
        }
    "#;
    assert!(!is_angular_file(src), "Comment with @Component should not trigger detection");
}

#[test]
fn rejects_string_literal_with_decorator_name() {
    // A-11: String literals should not trigger Angular detection
    let src = r#"
        const message = "Use @Component to define a component";
        export class SampleService {
            private isReady: boolean = false;
        }
    "#;
    assert!(!is_angular_file(src), "String literal with @Component should not trigger detection");
}

#[test]
fn rejects_react_component() {
    let src = r#"
        import React from 'react';
        export class MyComponent extends React.Component {
            render() { return <div />; }
        }
    "#;
    assert!(!is_angular_file(src));
}

#[test]
fn rejects_vue_component() {
    let src = r#"
        @Component
        export default {
            data() { return { count: 0 }; }
        }
    "#;
    // Note: Vue's `@Component` (without parentheses) is a decorator
    // form — the Angular detector requires `@Component(` with the
    // paren to be a strong signal. The Vue form has no parens so
    // the strong signal does not match.
    //
    // This is intentional: it avoids false positives on
    // similarly-named decorators in non-Angular frameworks.
    assert!(!is_angular_file(src));
}

#[test]
fn rejects_mobx_input_decorator() {
    // MobX uses `@Input` as a plain decorator without `@angular/core`.
    let src = r#"
        import { Input } from 'mobx-react';
        class Store { @Input value: string = ''; }
    "#;
    assert!(!is_angular_file(src));
}

#[test]
fn accepts_empty_string() {
    assert!(!is_angular_file(""));
}

#[test]
fn accepts_plain_text() {
    assert!(!is_angular_file("just some text without code"));
}
