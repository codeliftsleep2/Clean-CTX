// src/tests/angular_meta/mod.rs
//
// Integration tests for the Angular Meta-Layer Tier 1 entry point.

mod e2e;

use crate::angular_meta::run_meta_layer;
use crate::compression::Fidelity;

#[test]
fn meta_layer_extracts_inline_template_shape() {
    let source = r#"
        import { Component } from '@angular/core';

        @Component({
            selector: 'app-hello',
            template: '<div *ngIf="show"><span>{{ name }}</span></div>'
        })
        export class HelloComponent {
            name = 'World';
            show = true;
        }
    "#;
    let class_captures = vec![
        r#"
        @Component({
            selector: 'app-hello',
            template: '<div *ngIf="show"><span>{{ name }}</span></div>'
        })
        export class HelloComponent {
            name = 'World';
            show = true;
        }
    "#
        .to_string(),
    ];
    let block =
        run_meta_layer(source, &class_captures, Fidelity::Medium).expect("should detect Angular");
    let rendered = block.render();
    // Should contain the Φcmp: marker
    assert!(
        rendered.contains("Φcmp:HelloComponent"),
        "missing Φcmp marker: {}",
        rendered
    );
    // Should contain the inline template shape extraction
    assert!(
        rendered.contains("Φtpl:"),
        "missing Φtpl marker for inline template: {}",
        rendered
    );
    // Medium fidelity = multi-line structural output. The `Φtpl:` header
    // line is bare; the structural lines follow it. The template has
    // *ngIf (condition captured), and the div/span are HTML scaffolding
    // (stripped at Medium fidelity).
    let tpl_block: Vec<&str> = rendered
        .lines()
        .filter(|l| {
            l.starts_with("Φtpl:") || l.starts_with('@') || l.starts_with('*') || l.starts_with('<')
        })
        .collect();
    assert!(
        tpl_block
            .iter()
            .any(|l| l.contains("[ngIf]") || l.contains("*ngIf")),
        "missing [ngIf] directive in template output: {:?}",
        tpl_block
    );
    assert!(
        tpl_block.iter().any(|l| l.contains("show")),
        "missing *ngIf condition in template output: {:?}",
        tpl_block
    );
}

#[test]
fn meta_layer_returns_none_for_plain_typescript() {
    let src = r#"
        export class SampleService {
            private isReady: boolean = false;
            public async process(payload: string[]): Promise<boolean> {
                return true;
            }
        }
    "#;
    let class_captures = vec![src.to_string()];
    let result = run_meta_layer(src, &class_captures, Fidelity::Low);
    assert!(result.is_none(), "non-Angular file should return None");
}

#[test]
fn meta_layer_emits_phi_block_for_component() {
    let src = r#"
        @Component({
            selector: 'app-user-card',
            templateUrl: './user-card.component.html',
            styleUrls: ['./user-card.component.scss']
        })
        export class UserCardComponent {
            @Input() userId: string = '';
            @Input() userName: string = '';
            @Output() userDeleted = new EventEmitter<string>();
        }
    "#;
    let class_captures = vec![src.to_string()];
    // F-ANG-23: use Medium fidelity so field-level @Input/@Output
    // markers are emitted.
    let result = run_meta_layer(src, &class_captures, Fidelity::Medium);
    let block = result.expect("Angular component should produce a MetaBlock");
    let rendered = block.render();
    assert!(rendered.contains("// --- Φ Angular Meta ---"));
    assert!(rendered.contains("Φcmp:UserCardComponent"));
    assert!(rendered.contains("sel=app-user-card"));
    assert!(rendered.contains("tpl=./user-card.component.html"));
}

#[test]
fn meta_layer_emits_phi_block_for_service() {
    let src = r#"
        @Injectable({ providedIn: 'root' })
        export class UserService {
            constructor(private http: HttpClient) {}
        }
    "#;
    let class_captures = vec![src.to_string()];
    // F-ANG-23: Φinjects: is High-fidelity only.
    let result = run_meta_layer(src, &class_captures, Fidelity::High);
    let block = result.expect("Angular service should produce a MetaBlock");
    let rendered = block.render();
    assert!(rendered.contains("Φsvc:UserService"));
    assert!(rendered.contains("scope=root"));
    assert!(rendered.contains("Φinjects:[HttpClient]"));
}

#[test]
fn meta_layer_block_is_empty_when_no_decorators_match() {
    // File imports @angular/core but has no Angular decorator
    // (e.g. an interface-only file).
    let src = r#"
        import { Component } from '@angular/core';
        export interface UserCardData {
            userId: string;
            userName: string;
        }
    "#;
    let class_captures: Vec<String> = vec![];
    let result = run_meta_layer(src, &class_captures, Fidelity::Low);
    // No class captures → no MetaBlock.
    assert!(result.is_none());
}
