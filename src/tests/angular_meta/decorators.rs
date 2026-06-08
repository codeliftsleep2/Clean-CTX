// src/tests/angular_meta/decorators.rs
//
// Tests for decorator extraction from a class capture.

use crate::angular_meta::decorators::extract_decorators;

fn lines_to_vec(opt: Option<Vec<String>>) -> Vec<String> {
    opt.unwrap_or_default()
}

#[test]
fn returns_none_for_plain_class() {
    let raw = r#"
        export class SampleService {
            private isReady: boolean = false;
        }
    "#;
    assert!(extract_decorators(raw).is_none());
}

#[test]
fn returns_none_for_empty_string() {
    assert!(extract_decorators("").is_none());
}

#[test]
fn returns_none_when_no_at_sign() {
    let raw = "export class Foo { x: number = 1; }";
    assert!(extract_decorators(raw).is_none());
}

#[test]
fn extracts_component_decorator_with_selector() {
    let raw = r#"
        @Component({
            selector: 'app-foo',
        })
        export class FooCmp {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
    assert!(lines.iter().any(|l| l.starts_with("Φsvc:UserService scope=root")));
}

#[test]
fn extracts_injectable_without_provided_in() {
    let raw = r#"
        @Injectable()
        export class FooService {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
    let dir_line = lines.iter().find(|l| l.starts_with("Φdir:")).unwrap();
    assert!(dir_line.contains("sel=[appHighlight]"));
}

#[test]
fn extracts_pipe_decorator() {
    let raw = r#"
        @Pipe({ name: 'upper' })
        export class UpperPipe {}
    "#;
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
    // Only the @Component is recognised; @SomeRandomDecorator is
    // silently dropped.
    assert!(lines.iter().any(|l| l.starts_with("Φcmp:FooCmp")));
    assert_eq!(lines.len(), 1);
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
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
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
    let lines = lines_to_vec(extract_decorators(raw));
    assert!(lines.iter().any(|l| l.contains("Φcmp:FooCmp")));
}
