// src/tests/angular_meta/e2e.rs
//
// E2E Meta-Layer Tests — Issue #4 (FAANG Audit Remediation)
// Verifies the full pipeline: raw Angular source → meta-layer extraction → Φ markers
// with proper abbreviations (Φcmp:, Φsvc:, Φdir:, etc.) and no old-format markers.

use crate::angular_meta::run_meta_layer;
use crate::compression::Fidelity;

// ── Component E2E ──────────────────────────────────

#[test]
fn angular_component_to_phi_markers_e2e() {
    let source = r#"
        import { Component } from '@angular/core';

        @Component({
            selector: 'app-root',
            template: '<div>Hello {{name}}</div>',
            styles: ['h1 { color: red; }']
        })
        export class AppComponent {
            name = 'World';
            show = true;
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect Angular component");
    let rendered = block.render();

    assert!(rendered.contains("Φcmp:AppComponent"), "should contain Φcmp marker\n{}", rendered);
    assert!(rendered.contains("sel=app-root"), "should contain selector\n{}", rendered);
    assert!(!rendered.contains("NG_COMPONENT_"), "should not contain old NG_ prefix\n{}", rendered);
}

#[test]
fn angular_component_with_external_template_e2e() {
    let source = r#"
        import { Component, Input } from '@angular/core';

        @Component({
            selector: 'app-user-card',
            templateUrl: './user-card.component.html',
            styleUrls: ['./user-card.component.scss']
        })
        export class UserCardComponent {
            @Input() userId: string = '';
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect Angular component");
    let rendered = block.render();

    assert!(rendered.contains("Φcmp:UserCardComponent"), "should contain Φcmp marker\n{}", rendered);
    assert!(rendered.contains("sel=app-user-card"), "should contain selector\n{}", rendered);
    assert!(rendered.contains("tpl=./user-card.component.html"), "should contain template path\n{}", rendered);
}

// ── Service E2E ────────────────────────────────────

#[test]
fn angular_service_to_phi_markers_e2e() {
    let source = r#"
        import { Injectable } from '@angular/core';

        @Injectable({
            providedIn: 'root'
        })
        export class UserService {
            constructor(private http: any) {}
            getUsers() { return []; }
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect Angular service");
    let rendered = block.render();

    assert!(rendered.contains("Φsvc:UserService"), "should contain Φsvc marker\n{}", rendered);
    assert!(!rendered.contains("NG_COMPONENT_"), "should not contain old NG_ prefix\n{}", rendered);
}

// ── Injectable E2E ─────────────────────────────────

#[test]
fn angular_injectable_to_phi_markers_e2e() {
    let source = r#"
        import { Injectable } from '@angular/core';

        @Injectable()
        export class LoggerService {
            log(msg: string) { console.log(msg); }
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect injectable");
    let rendered = block.render();

    // @Injectable() without providedIn produces Φsvc marker
    assert!(rendered.contains("Φsvc:LoggerService"), "should contain Φsvc marker\n{}", rendered);
}

// ── Directive E2E ──────────────────────────────────

#[test]
fn angular_directive_to_phi_markers_e2e() {
    let source = r#"
        import { Directive, ElementRef } from '@angular/core';

        @Directive({
            selector: '[appHighlight]'
        })
        export class HighlightDirective {
            constructor(private el: any) {}
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect directive");
    let rendered = block.render();

    assert!(rendered.contains("Φdir:HighlightDirective"), "should contain Φdir marker\n{}", rendered);
    assert!(rendered.contains("sel=[appHighlight]"), "should contain directive selector\n{}", rendered);
}

// ── Pipe E2E ───────────────────────────────────────

#[test]
fn angular_pipe_to_phi_markers_e2e() {
    let source = r#"
        import { Pipe, PipeTransform } from '@angular/core';

        @Pipe({
            name: 'capitalize'
        })
        export class CapitalizePipe implements PipeTransform {
            transform(value: string): string {
                return value.charAt(0).toUpperCase() + value.slice(1);
            }
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect pipe");
    let rendered = block.render();

    assert!(rendered.contains("Φpipe:CapitalizePipe"), "should contain Φpipe marker\n{}", rendered);
    assert!(rendered.contains("name=capitalize"), "should contain pipe name\n{}", rendered);
}

// ── NgModule E2E ───────────────────────────────────

#[test]
fn angular_ngmodule_to_phi_markers_e2e() {
    let source = r#"
        import { NgModule } from '@angular/core';
        import { BrowserModule } from '@angular/platform-browser';
        import { AppComponent } from './app.component';

        @NgModule({
            declarations: [AppComponent],
            imports: [BrowserModule],
            bootstrap: [AppComponent]
        })
        export class AppModule { }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect NgModule");
    let rendered = block.render();

    assert!(rendered.contains("Φmod:AppModule"), "should contain Φmod marker\n{}", rendered);
}

// ── Non-Angular file (negative test) ───────────────

#[test]
fn plain_typescript_no_angular_markers_e2e() {
    let source = r#"
        export class MathUtils {
            static add(a: number, b: number): number {
                return a + b;
            }
        }
    "#;
    let class_captures = vec![source.to_string()];

    let result = run_meta_layer(source, &class_captures, Fidelity::Medium);
    assert!(result.is_none(), "plain TS should not produce Angular meta-block");
}

// ── No old format regression ──────────────────────

#[test]
fn angular_e2e_no_old_format_markers() {
    let source = r#"
        import { Component } from '@angular/core';

        @Component({ selector: 'app-test', template: '<div></div>' })
        export class TestComponent {
            name = 'test';
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Medium)
        .expect("should detect Angular component");
    let rendered = block.render();

    // Verify no old format markers anywhere
    let old_formats = ["NG_COMPONENT_", "NG_SERVICE_", "NG_DIRECTIVE_", "NG_PIPE_", "NG_MODULE_"];
    for old in &old_formats {
        assert!(!rendered.contains(old), "should not contain old format {}\n{}", old, rendered);
    }

    // Verify Φ abbreviations use new format (not old NG_ prefix)
    assert!(rendered.contains("Φcmp:TestComponent"), "should contain Φcmp\n{}", rendered);
}

// ── Low fidelity still emits base markers ──────────

#[test]
fn angular_low_fidelity_emits_base_phi_markers_e2e() {
    let source = r#"
        import { Component } from '@angular/core';

        @Component({
            selector: 'app-root',
            template: '<div>Hello</div>'
        })
        export class AppComponent {
            name = 'World';
        }
    "#;
    let class_captures = vec![source.to_string()];

    let block = run_meta_layer(source, &class_captures, Fidelity::Low)
        .expect("should detect Angular component at Low fidelity");
    let rendered = block.render();

    // Low fidelity still emits base markers (no template shapes, no injects)
    assert!(rendered.contains("Φcmp:AppComponent"), "should contain Φcmp at Low fidelity\n{}", rendered);
    assert!(!rendered.contains("NG_COMPONENT_"), "should not contain old NG_ prefix\n{}", rendered);
}