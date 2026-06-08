// src/tests/angular_meta/mod.rs
//
// Integration tests for the Angular Meta-Layer Tier 1 entry point.

use crate::angular_meta::run_meta_layer;
use crate::compression::Fidelity;

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
    let result = run_meta_layer(src, &class_captures, Fidelity::Low);
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
    let result = run_meta_layer(src, &class_captures, Fidelity::Low);
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
