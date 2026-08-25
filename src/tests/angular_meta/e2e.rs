// src/tests/angular_meta/e2e.rs
//
// E2E Meta-Layer Tests — Issue #4 (FAANG Audit Remediation)
// Verifies the full pipeline: raw Angular source → meta-layer extraction → Φ markers
// with proper abbreviations (Φcmp:, Φsvc:, Φdir:, etc.) and no old-format markers.

use crate::angular_meta::run_meta_layer;
use crate::angular_meta::run_meta_layer_with_config;
use crate::compression::Fidelity;

// ── Component E2E ──────────────────────────────────

// ── Config-Threading Regression (Deep Audit Phase F) ──────────────
//
// Verifies the `MetaLayerConfig` actually reaches the pipeline. Prior to
// the deep audit, `run_meta_layer` (the backward-compat variant) was
// always called — the `enabled` flags and sub-layer settings in
// `.clean-ctx.json` were silently ignored. This test would have failed
// before the config was threaded through `MetaLayer::enrich` →
// `run_meta_layer_with_config`.

#[test]
fn disabled_rxjs_ngrx_produces_zero_markers() {
    use crate::config::{MetaLayerConfig, NgRxConfig, RoutingConfig, RxJsConfig, SignalsConfig};

    let source = r#"
        import { Injectable } from '@angular/core';
        import { Observable, BehaviorSubject, of, from } from 'rxjs';
        import { map, switchMap, catchError } from 'rxjs/operators';
        import { createAction, createReducer, on, createSelector } from '@ngrx/store';

        const loadUsers = createAction('[User] Load Users', (u: any) => ({ u }));
        const loadUsersSuccess = createAction('[User] Load Users Success');

        @Injectable({ providedIn: 'root' })
        export class UserService {
            users$: Observable<User[]> = of([]);
            private trigger$ = new BehaviorSubject<void>(undefined);
            load() {
                return this.trigger$.pipe(
                    switchMap(() => of([])),
                    map(users => users),
                    catchError(err => of([]))
                );
            }
        }
    "#;
    let class_captures = vec![source.to_string()];

    // Disable RxJS and NgRx sub-layers — they must NOT emit anything.
    // Keep signals/routing enabled (defaults) so we prove only the
    // disabled layers are suppressed.
    let cfg = MetaLayerConfig {
        rxjs: RxJsConfig {
            enabled: false,
            ..Default::default()
        },
        ngrx: NgRxConfig {
            enabled: false,
            ..Default::default()
        },
        signals: SignalsConfig::default(),
        routing: RoutingConfig::default(),
        ..Default::default()
    };

    let block = run_meta_layer_with_config(source, &class_captures, Fidelity::Medium, Some(&cfg))
        .expect("should still detect Angular @Injectable");
    let rendered = block.render();

    // The Angular decorator block is still emitted.
    assert!(
        rendered.contains("Φsvc:UserService"),
        "should contain Φsvc marker\n{}",
        rendered
    );
    // Disabled layers must produce zero markers.
    assert!(
        !rendered.contains("Φobs:"),
        "disabled RxJS must not emit Φobs:\n{}",
        rendered
    );
    assert!(
        !rendered.contains("Φsubject:"),
        "disabled RxJS must not emit Φsubject:\n{}",
        rendered
    );
    assert!(
        !rendered.contains("Φmap:"),
        "disabled RxJS must not emit Φmap:\n{}",
        rendered
    );
    assert!(
        !rendered.contains("Φcatch:"),
        "disabled RxJS must not emit Φcatch:\n{}",
        rendered
    );
    assert!(
        !rendered.contains("Φaction:"),
        "disabled NgRx must not emit Φaction:\n{}",
        rendered
    );
}

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

    assert!(
        rendered.contains("Φcmp:AppComponent"),
        "should contain Φcmp marker\n{}",
        rendered
    );
    assert!(
        rendered.contains("sel=app-root"),
        "should contain selector\n{}",
        rendered
    );
    assert!(
        !rendered.contains("NG_COMPONENT_"),
        "should not contain old NG_ prefix\n{}",
        rendered
    );
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

    assert!(
        rendered.contains("Φcmp:UserCardComponent"),
        "should contain Φcmp marker\n{}",
        rendered
    );
    assert!(
        rendered.contains("sel=app-user-card"),
        "should contain selector\n{}",
        rendered
    );
    assert!(
        rendered.contains("tpl=./user-card.component.html"),
        "should contain template path\n{}",
        rendered
    );
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

    assert!(
        rendered.contains("Φsvc:UserService"),
        "should contain Φsvc marker\n{}",
        rendered
    );
    assert!(
        !rendered.contains("NG_COMPONENT_"),
        "should not contain old NG_ prefix\n{}",
        rendered
    );
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
    assert!(
        rendered.contains("Φsvc:LoggerService"),
        "should contain Φsvc marker\n{}",
        rendered
    );
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

    let block =
        run_meta_layer(source, &class_captures, Fidelity::Medium).expect("should detect directive");
    let rendered = block.render();

    assert!(
        rendered.contains("Φdir:HighlightDirective"),
        "should contain Φdir marker\n{}",
        rendered
    );
    assert!(
        rendered.contains("sel=[appHighlight]"),
        "should contain directive selector\n{}",
        rendered
    );
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

    let block =
        run_meta_layer(source, &class_captures, Fidelity::Medium).expect("should detect pipe");
    let rendered = block.render();

    assert!(
        rendered.contains("Φpipe:CapitalizePipe"),
        "should contain Φpipe marker\n{}",
        rendered
    );
    assert!(
        rendered.contains("name=capitalize"),
        "should contain pipe name\n{}",
        rendered
    );
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

    let block =
        run_meta_layer(source, &class_captures, Fidelity::Medium).expect("should detect NgModule");
    let rendered = block.render();

    assert!(
        rendered.contains("Φmod:AppModule"),
        "should contain Φmod marker\n{}",
        rendered
    );
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
    assert!(
        result.is_none(),
        "plain TS should not produce Angular meta-block"
    );
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
    let old_formats = [
        "NG_COMPONENT_",
        "NG_SERVICE_",
        "NG_DIRECTIVE_",
        "NG_PIPE_",
        "NG_MODULE_",
    ];
    for old in &old_formats {
        assert!(
            !rendered.contains(old),
            "should not contain old format {}\n{}",
            old,
            rendered
        );
    }

    // Verify Φ abbreviations use new format (not old NG_ prefix)
    assert!(
        rendered.contains("Φcmp:TestComponent"),
        "should contain Φcmp\n{}",
        rendered
    );
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
    assert!(
        rendered.contains("Φcmp:AppComponent"),
        "should contain Φcmp at Low fidelity\n{}",
        rendered
    );
    assert!(
        !rendered.contains("NG_COMPONENT_"),
        "should not contain old NG_ prefix\n{}",
        rendered
    );
}

// ── Phase C: Structured MetaLayerOutput (no render-then-reparse) ──
//
// The deep-audit remediation replaced the render-then-reparse anti-pattern:
// `MetaLayer::enrich` previously wrote a rendered `String` which the
// pipeline re-parsed back into a structured `MetaBlock` by string-splitting
// on `// ---` headers. Now `enrich` returns a structured `MetaLayerOutput`
// (structured block + rendered text) and the pipeline consumes the
// structured block directly.
//
// This test drives the full registry path and verifies:
//   1. The structured Angular block carries decorator lines + named
//      sections (RxJS), with no cross-contamination.
//   2. The rendered text matches `MetaBlock::render()` exactly — i.e.
//      the structured form and the string form are consistent (no
//      information is lost in either direction).

#[test]
fn structured_meta_layer_output_preserves_angular_sections() {
    let source = r#"
        import { Injectable } from '@angular/core';
        import { BehaviorSubject, Observable, of } from 'rxjs';
        import { map, switchMap } from 'rxjs/operators';

        @Injectable({ providedIn: 'root' })
        export class UserService {
            users$: Observable<string[]> = of([]);
            private refresh$ = new BehaviorSubject<void>(undefined);
            load() {
                return this.refresh$.pipe(
                    switchMap(() => of([])),
                    map((x: string[]) => x)
                );
            }
        }
    "#;

    let registry = crate::layers::LayerRegistry::global();
    let results =
        registry.run_meta_layers_pipeline(source, &[source.to_string()], Fidelity::Medium, None);

    // The angular layer must have emitted a structured output.
    let angular = results
        .iter()
        .find(|o| o.layer_name == "angular")
        .expect("angular layer should produce output for this source");

    // Structured block is present and carries the decorator line.
    let block = angular
        .angular_block
        .as_ref()
        .expect("angular output must carry a structured MetaBlock");
    assert!(
        block.lines.iter().any(|l| l.contains("Φsvc:UserService")),
        "structured block should contain Φsvc line, got: {:?}",
        block.lines
    );

    // The RxJS section is a named section (not leaked into the decorator lines).
    let rx_section = block
        .sections
        .iter()
        .find(|s| s.header.contains("RxJS"))
        .expect("structured block should contain an RxJS named section");
    assert!(
        rx_section
            .lines
            .iter()
            .any(|l| l.trim_start().starts_with("Φsubject:refresh$")),
        "RxJS section should contain Φsubject:refresh$ line, got: {:?}",
        rx_section.lines
    );
    assert!(
        rx_section
            .lines
            .iter()
            .any(|l| l.trim_start().starts_with("Φobs:users$")),
        "RxJS section should contain Φobs:users$ line, got: {:?}",
        rx_section.lines
    );

    // The rendered text in MetaLayerOutput must match MetaBlock::render()
    // exactly — this is the invariant that makes the structured form and
    // the string form interchangeable (no information loss).
    let block_rendered = block.render();
    assert_eq!(
        angular.rendered, block_rendered,
        "MetaLayerOutput.rendered must equal MetaBlock::render()"
    );
    assert!(
        block_rendered.contains("// --- Φ RxJS Meta ---"),
        "rendered text should contain the RxJS section header"
    );
}

/// The pipeline's `build_output_lines` must assign the structured blocks
/// directly — a regression that fails if the render-then-reparse path is
/// reintroduced (blocks would no longer be sectioned correctly).
#[test]
fn build_output_lines_assigns_structured_meta_blocks() {
    let source = r#"
        import { Injectable } from '@angular/core';
        import { Observable, of } from 'rxjs';
        import { map } from 'rxjs/operators';

        @Injectable({ providedIn: 'root' })
        export class DataService {
            items$: Observable<string[]> = of([]);
            fetch() {
                return this.items$.pipe(map((x) => x));
            }
        }
    "#;

    // Use the compression pipeline directly. `skip_set` = None,
    // `config` = None (defaults).
    // The decorator extractor walks each class capture's text looking for
    // `@Injectable` immediately preceding the class — so the capture text
    // must carry the full source (matching the other e2e tests).
    let all_captures = vec![crate::compression::CapEntry {
        name: "class.root".to_string(),
        text: source.to_string(),
        raw_text: source.to_string(),
        start_byte: 0,
        end_byte: source.len(),
    }];
    let built = crate::compression::pipeline::build_output_lines(
        &all_captures,
        source,
        Fidelity::Medium,
        None,
        None,
    );

    // Angular leaf block must exist and carry both decorator lines and a
    // named RxJS section — proving the structured form survived the
    // pipeline (no string-splitting re-parse happened).
    let block = built
        .meta_block
        .as_ref()
        .expect("build_output_lines should attach the angular MetaBlock");
    assert!(
        block.lines.iter().any(|l| l.contains("Φsvc:DataService")),
        "MetaBlock.lines should carry the decorator line, got: {:?}",
        block.lines
    );
    assert!(
        block.sections.iter().any(|s| s.header.contains("RxJS")),
        "MetaBlock.sections should carry the RxJS section, got: {:?}",
        block.sections
    );
}
