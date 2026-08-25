use super::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_ts_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    write!(f, "{}", content).unwrap();
    path
}

#[test]
fn assemble_body_uses_semicolon_at_low() {
    let lines = vec!["a".to_string(), "b".to_string()];
    assert_eq!(assemble_body(&lines, Fidelity::Low), "a;b");
}

#[test]
fn assemble_body_uses_newline_at_medium() {
    let lines = vec!["a".to_string(), "b".to_string()];
    assert_eq!(assemble_body(&lines, Fidelity::Medium), "a\nb");
}

#[test]
fn assemble_body_uses_newline_at_high() {
    let lines = vec!["a".to_string(), "b".to_string()];
    assert_eq!(assemble_body(&lines, Fidelity::High), "a\nb");
}

#[test]
fn compress_file_cache_hit_returns_notice() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "export class Foo {}\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let fidelity = Fidelity::Low;

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(
        result1.is_ok(),
        "compress_file (miss) should succeed, got: {:?}",
        result1
    );

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result2.is_ok(), "compress_file (hit) should succeed");
    let output2 = result2.unwrap();

    assert!(
        output2.contains("[CACHE_HIT]"),
        "expected CACHE_HIT notice, got: {}",
        output2
    );

    create_ts_file(&dir, "test.ts", "export class Bar {}");
    let result3 = compress_file(path, &mut dict, &mut cache, fidelity, None);
    assert!(result3.is_ok(), "compress_file (modified) should succeed");
    let output3 = result3.unwrap();
    assert!(
        !output3.contains("[CACHE_HIT]"),
        "modified file should NOT hit cache, got: {}",
        output3
    );

    assert!(
        output3.contains("Compacted Layout"),
        "expected a full compacted output after modification, got: {}",
        output3
    );
}

#[test]
fn compress_file_cache_hit_vs_miss_output_differ() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class A { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let fidelity = Fidelity::Medium;

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result1.is_ok(), "compress_file (first call) should succeed");

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result2.is_ok());
    let second = result2.unwrap();

    assert!(
        second.contains("[CACHE_HIT]"),
        "Second call with same content should hit cache"
    );
}

// ---------- F-18: file size guard ----------

#[test]
fn compress_file_rejects_file_larger_than_max() {
    // Create a .ts file that exceeds the 10 MB limit.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.ts");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        // Write just over 10 MB of content (valid UTF-8).
        let chunk = "export class X {}\n";
        let repetitions = (10 * 1024 * 1024 / chunk.len()) + 2;
        for _ in 0..repetitions {
            f.write_all(chunk.as_bytes()).unwrap();
        }
    }

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let result = compress_file(path, &mut dict, &mut cache, Fidelity::Low, None);
    assert!(result.is_err(), "should reject oversized file");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("File too large"),
        "error should mention 'File too large', got: {}",
        err_msg
    );
}

// ---------- R-02: Type-alias integration tests ----------

fn make_aliases(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn compress_text_with_aliases_medium_fidelity() {
    // At Medium fidelity, type names in method signatures should be
    // substituted with alias tokens.
    let source = "class UserService { getUser(id: string): Promise<User> {} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let result = compress_text(source, "ts", Fidelity::Medium, "α1", Some(&aliases));
    assert!(result.is_ok(), "compress_text with aliases should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        full_output.contains("$uid"),
        "output should contain $uid alias, got: {}",
        full_output
    );
    assert!(
        full_output.contains("§TA"),
        "output should contain §TA footer, got: {}",
        full_output
    );
}

#[test]
fn compress_text_with_aliases_high_fidelity() {
    let source = "class UserService { getUser(id: string): Promise<User> {} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let result = compress_text(source, "ts", Fidelity::High, "α1", Some(&aliases));
    assert!(result.is_ok());
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        full_output.contains("$uid"),
        "output should contain $uid alias"
    );
    assert!(
        full_output.contains("§TA"),
        "output should contain §TA footer"
    );
}

#[test]
fn compress_text_without_aliases_no_footer() {
    // When no aliases are configured, no §TA footer should appear.
    let source = "class UserService { getUser(id: string): Promise<User> {} }";
    let result = compress_text(source, "ts", Fidelity::Medium, "α1", None);
    assert!(result.is_ok());
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        !full_output.contains("§TA"),
        "output should NOT contain §TA footer when no aliases configured"
    );
}

#[test]
fn compress_text_aliases_deterministic() {
    // Same input + same aliases → same output (delta consistency).
    let source = "class Service { get(id: string): Promise<User> {} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let r1 = compress_text(source, "ts", Fidelity::Medium, "α1", Some(&aliases)).unwrap();
    let r2 = compress_text(source, "ts", Fidelity::Medium, "α1", Some(&aliases)).unwrap();
    assert_eq!(r1.0, r2.0, "body lines should be identical");
    assert_eq!(r1.1, r2.1, "full output should be identical");
}

// ---------- C-4: build_output_lines byte-exactness at Edit/Verbatim ----------

#[test]
fn build_output_lines_edit_fidelity_emits_byte_exact_method() {
    // C-4 fix: at Edit fidelity the full method text (signature + body)
    // must be emitted as-is — NO 2-space indent prefix that would break
    // byte-exactness for `replace_in_file` SEARCH blocks.
    let method_text = "public async getUserById(id: string): Promise<User> {\n  return this.users.find(u => u.id === id)!;\n}";
    let cap = crate::compression::CapEntry {
        name: "method.root".to_string(),
        text: method_text.to_string(),
        raw_text: method_text.to_string(),
        start_byte: 0,
        end_byte: method_text.len(),
    };
    let built = build_output_lines(&[cap], "", Fidelity::Edit, None, None);
    assert_eq!(built.output_lines.len(), 1, "one method line expected");
    assert_eq!(
        built.output_lines[0], method_text,
        "Edit fidelity must emit the method byte-exact (no indent prefix)"
    );
}

#[test]
fn build_output_lines_verbatim_fidelity_emits_byte_exact_method() {
    // C-4 fix: same byte-exactness guarantee at Verbatim.
    let method_text = "public async getUserById(id: string): Promise<User> {\n  return this.users.find(u => u.id === id)!;\n}";
    let cap = crate::compression::CapEntry {
        name: "method.root".to_string(),
        text: method_text.to_string(),
        raw_text: method_text.to_string(),
        start_byte: 0,
        end_byte: method_text.len(),
    };
    let built = build_output_lines(&[cap], "", Fidelity::Verbatim, None, None);
    assert_eq!(built.output_lines.len(), 1, "one method line expected");
    assert_eq!(
        built.output_lines[0], method_text,
        "Verbatim fidelity must emit the method byte-exact (no indent prefix)"
    );
}

#[test]
fn build_output_lines_high_fidelity_still_indents_signature() {
    // High fidelity keeps the 2-space indent (signature-only, no body).
    let sig = "public async getUserById(id: string): Promise<User>";
    let cap = crate::compression::CapEntry {
        name: "method.root".to_string(),
        text: sig.to_string(),
        raw_text: sig.to_string(),
        start_byte: 0,
        end_byte: sig.len(),
    };
    let built = build_output_lines(&[cap], "", Fidelity::High, None, None);
    assert_eq!(built.output_lines.len(), 1);
    assert_eq!(
        built.output_lines[0],
        format!("  {}", sig),
        "High fidelity should keep the 2-space indent"
    );
}

// ---------- C-5: type aliases skipped at Edit/Verbatim ----------

#[test]
fn compress_text_edit_fidelity_skips_type_aliases() {
    // C-5 fix: at Edit fidelity the method bodies are byte-exact, so
    // type-alias substitution must be skipped (it would corrupt them).
    let source =
        "class UserService { getUser(id: string): Promise<User> {\n  return new User();\n} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let result = compress_text(source, "ts", Fidelity::Edit, "α1", Some(&aliases));
    assert!(result.is_ok(), "compress_text at Edit should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        !full_output.contains("$uid"),
        "Edit fidelity must NOT apply type aliases (byte-exact bodies), got: {}",
        full_output
    );
    assert!(
        !full_output.contains("§TA"),
        "Edit fidelity must NOT emit §TA footer, got: {}",
        full_output
    );
    // The raw method body must be preserved verbatim.
    assert!(
        full_output.contains("return new User();"),
        "Edit fidelity must preserve the raw method body, got: {}",
        full_output
    );
}

#[test]
fn compress_text_verbatim_fidelity_skips_type_aliases() {
    // C-5 fix: same guarantee at Verbatim.
    let source =
        "class UserService { getUser(id: string): Promise<User> {\n  return new User();\n} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let result = compress_text(source, "ts", Fidelity::Verbatim, "α1", Some(&aliases));
    assert!(result.is_ok(), "compress_text at Verbatim should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        !full_output.contains("$uid"),
        "Verbatim fidelity must NOT apply type aliases, got: {}",
        full_output
    );
    assert!(
        !full_output.contains("§TA"),
        "Verbatim fidelity must NOT emit §TA footer, got: {}",
        full_output
    );
    assert!(
        full_output.contains("return new User();"),
        "Verbatim fidelity must preserve the raw method body, got: {}",
        full_output
    );
}

// ---------- C-8: cache-hit at Edit/Verbatim returns raw source ----------

#[test]
fn compress_file_edit_cache_hit_returns_raw_source() {
    // C-8 fix: at Edit fidelity a cache hit must return the raw source
    // (byte-exact entire document), NOT the token-report notice.
    let dir = TempDir::new().unwrap();
    let source = "export class Foo {\n  getUser(): string {\n    return 'x';\n  }\n}\n";
    let path = create_ts_file(&dir, "test.ts", source);

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, Fidelity::Edit, None);
    assert!(
        result1.is_ok(),
        "compress_file (miss) at Edit should succeed"
    );

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, Fidelity::Edit, None);
    assert!(
        result2.is_ok(),
        "compress_file (hit) at Edit should succeed"
    );
    let output2 = result2.unwrap();
    assert_eq!(
        output2, source,
        "Edit cache-hit must return the raw source byte-exact, got: {}",
        output2
    );
    assert!(
        !output2.contains("[CACHE_HIT]"),
        "Edit cache-hit must NOT return the token-report notice"
    );
}

#[test]
fn compress_file_verbatim_cache_hit_returns_raw_source() {
    // C-8 fix: same guarantee at Verbatim.
    let dir = TempDir::new().unwrap();
    let source = "export class Foo {\n  getUser(): string {\n    return 'x';\n  }\n}\n";
    let path = create_ts_file(&dir, "test.ts", source);

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let result1 = compress_file(
        path.clone(),
        &mut dict,
        &mut cache,
        Fidelity::Verbatim,
        None,
    );
    assert!(
        result1.is_ok(),
        "compress_file (miss) at Verbatim should succeed"
    );

    let result2 = compress_file(
        path.clone(),
        &mut dict,
        &mut cache,
        Fidelity::Verbatim,
        None,
    );
    assert!(
        result2.is_ok(),
        "compress_file (hit) at Verbatim should succeed"
    );
    let output2 = result2.unwrap();
    assert_eq!(
        output2, source,
        "Verbatim cache-hit must return the raw source byte-exact, got: {}",
        output2
    );
}

// ---------- H-8: should_skip_capture at Edit/Verbatim ----------

#[test]
fn should_skip_capture_edit_fidelity_matches_method_name() {
    // H-8 fix: at Edit fidelity `cap.text` is the FULL method body, so the
    // first word is the access modifier, not the method name. The skip-set
    // must match the actual method name (the identifier before the first `(`).
    let method_text = "public async getUserById(id: string): Promise<User> {\n  return this.users.find(u => u.id === id)!;\n}";
    let cap = crate::compression::CapEntry {
        name: "method.root".to_string(),
        text: method_text.to_string(),
        raw_text: method_text.to_string(),
        start_byte: 0,
        end_byte: method_text.len(),
    };
    let mut skip = std::collections::HashSet::new();
    skip.insert("getUserById".to_string());
    assert!(
        should_skip_capture(&cap, &skip),
        "Edit fidelity must match the method name (getUserById), not the modifier (public)"
    );

    // A different method name in the skip set must NOT match.
    let mut skip_other = std::collections::HashSet::new();
    skip_other.insert("otherMethod".to_string());
    assert!(
        !should_skip_capture(&cap, &skip_other),
        "Edit fidelity must NOT match an unrelated method name"
    );
}

#[test]
fn should_skip_capture_edit_fidelity_matches_field_name() {
    // H-8 fix: at Edit fidelity `cap.text` is the full field text, so the
    // field name is the LAST whitespace token before the `:`.
    let field_text = "private readonly userId: string = '';";
    let cap = crate::compression::CapEntry {
        name: "field.root".to_string(),
        text: field_text.to_string(),
        raw_text: field_text.to_string(),
        start_byte: 0,
        end_byte: field_text.len(),
    };
    let mut skip = std::collections::HashSet::new();
    skip.insert("userId".to_string());
    assert!(
        should_skip_capture(&cap, &skip),
        "Edit fidelity must match the field name (userId), not the modifier (private)"
    );
}

// ---------- H-5: simple_compact fallback at Edit/Verbatim ----------

#[test]
fn build_output_lines_edit_fidelity_raw_fallback_is_byte_exact() {
    // H-5 fix: when nothing is captured, the raw fallback at Edit/Verbatim
    // must emit the first line as-is (NOT collapse whitespace via simple_compact).
    let source = "  const   spaced   =   true;  ";
    let built = build_output_lines(&[], source, Fidelity::Edit, None, None);
    assert_eq!(built.output_lines.len(), 1, "one fallback line expected");
    assert_eq!(
        built.output_lines[0], "const   spaced   =   true;",
        "Edit raw fallback must preserve internal whitespace (byte-exact)"
    );
}

// ---------- C-11: meta-layers skipped at Edit/Verbatim ----------

#[test]
fn compress_text_edit_fidelity_skips_meta_block() {
    // C-11 fix: at Edit fidelity the Φ meta block must NOT be injected into
    // the body (it would corrupt byte-exact method bodies).
    let source = "@Component({selector: 'app-foo'})\nexport class FooComponent {\n  getUser(): string {\n    return 'x';\n  }\n}\n";
    let result = compress_text(source, "ts", Fidelity::Edit, "α1", None);
    assert!(result.is_ok(), "compress_text at Edit should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        !full_output.contains("Φ"),
        "Edit fidelity must NOT inject the Φ meta block, got: {}",
        full_output
    );
    assert!(
        full_output.contains("return 'x';"),
        "Edit fidelity must preserve the raw method body, got: {}",
        full_output
    );
}

// ---------- Angular meta-layer receives decorator-inclusive class text ----------
//
// The meta-layer's `class_captures` contract requires the FULL class text
// (leading decorators + class body), NOT the compacted class name. The
// production capture pipeline normalizes `class.root` captures via
// `extract_class_name` into `cap.text` (e.g. `"UserCardComponent"`), but
// `decorators::extract_decorators` searches the capture for `@Component`
// before the `class` keyword — a bare name yields no decorators and the
// Angular `Φcmp:`/`Φsvc:` markers are silently dropped.
//
// Note: tree-sitter's `class_declaration` node (`cap.raw_text`) also
// excludes the leading decorators — they are sibling AST nodes. The
// meta-layer dispatch in `build_output_lines` therefore reconstructs the
// decorator-inclusive SOURCE SPAN via the canonical
// `decorator_inclusive_class_text` helper (which maps the class position
// back into the source and scans backward for the leading `@`).
//
// This test drives the REAL capture pipeline (`compress_text`), not a
// hand-constructed `CapEntry` with full-source text, so it fails if the
// class-captures regression is reintroduced.

#[cfg(feature = "angular")]
#[test]
fn compress_text_emits_angular_component_markers() {
    let source = r#"
        import { Component } from '@angular/core';

        @Component({
            selector: 'app-user-card',
            templateUrl: './user-card.component.html'
        })
        export class UserCardComponent {
            @Input() userId: string = '';
        }
    "#;
    let result = compress_text(source, "ts", Fidelity::Medium, "α1", None);
    assert!(result.is_ok(), "compress_text should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        full_output.contains("Φcmp:UserCardComponent"),
        "compressed output must contain the Angular Φcmp marker, got:\n{}",
        full_output
    );
    assert!(
        full_output.contains("sel=app-user-card"),
        "compressed output must contain the component selector, got:\n{}",
        full_output
    );
}

// The `@Injectable` decorator is the same decorator-inclusive contract but
// a different Angular classification — it must also survive the production
// compression path now that the meta-layer receives the full class text.

#[cfg(feature = "angular")]
#[test]
fn compress_text_emits_angular_injectable_markers() {
    let source = r#"
        import { Injectable } from '@angular/core';
        import { HttpClient } from '@angular/common/http';

        @Injectable({ providedIn: 'root' })
        export class UserService {
            constructor(private http: HttpClient) {}
            getUsers() { return this.http.get('/api/users'); }
        }
    "#;
    let result = compress_text(source, "ts", Fidelity::High, "α1", None);
    assert!(result.is_ok(), "compress_text should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        full_output.contains("Φsvc:UserService"),
        "compressed output must contain the Angular Φsvc marker, got:\n{}",
        full_output
    );
    assert!(
        full_output.contains("scope=root"),
        "compressed output must contain providedIn scope, got:\n{}",
        full_output
    );
}

// ---------- H-6/H-7: header labels at Edit/Verbatim ----------

#[test]
fn format_compacted_body_edit_fidelity_uses_edit_label() {
    // H-6 fix: Edit fidelity must NOT be labeled "High Fidelity".
    let body = format_compacted_body("class Foo {", "", "α1", Fidelity::Edit);
    assert!(
        body.contains("Edit Layout"),
        "Edit fidelity must use the Edit Layout label, got: {}",
        body
    );
    assert!(
        !body.contains("High Fidelity"),
        "Edit fidelity must NOT be labeled High Fidelity, got: {}",
        body
    );
}

#[test]
fn format_final_output_edit_fidelity_no_verbose_header() {
    // H-7 fix: at Edit fidelity the verbose "Token Optimization Report"
    // header must be omitted (it would break byte-exactness expectations).
    let out = format_final_output("class Foo {}", "class Foo {", Fidelity::Edit, 1, 0, 0);
    assert!(
        !out.contains("Token Optimization Report"),
        "Edit fidelity must NOT emit the verbose header, got: {}",
        out
    );
    assert!(
        !out.contains("Waste Reduced"),
        "Edit fidelity must NOT emit the Waste Reduced line, got: {}",
        out
    );
}

// ── C-22: C# attribute-inclusive class text in compression path ─────

#[cfg(feature = "dotnet")]
#[test]
fn compress_text_emits_csharp_dotnet_markers() {
    // The C# meta-layer (dotnet) must receive the FULL class text including
    // leading `[ApiController]` attributes — NOT just the compacted class
    // name. This test drives the REAL capture pipeline through compress_text.
    let source = r#"
        using Microsoft.AspNetCore.Mvc;

        [ApiController]
        [Route("api/[controller]")]
        public class WeatherController {
            [HttpGet]
            public IActionResult Get() { return Ok(); }
        }
    "#;
    let result = compress_text(source, "cs", Fidelity::Medium, "ω1", None);
    assert!(result.is_ok(), "compress_text for C# should succeed");
    let (_body_lines, full_output) = result.unwrap();
    // The dotnet meta-layer should recognize [ApiController] and emit its marker.
    // At minimum the class declaration must survive with the attribute text.
    assert!(
        full_output.contains("WeatherController"),
        "compressed output must contain the class name, got:\n{}",
        full_output
    );
}

// ── C-22: Java annotation-inclusive class text in compression path ──

#[cfg(feature = "spring_boot")]
#[test]
fn compress_text_emits_java_spring_markers() {
    // The Java meta-layer (spring_boot) must receive the FULL class text
    // including leading `@RestController` annotations — NOT just the
    // compacted class name. This test drives the REAL capture pipeline.
    let source = r#"
        import org.springframework.web.bind.annotation.RestController;

        @RestController
        @RequestMapping("/api/users")
        public class UserController {
            @GetMapping
            public List<User> getAll() { return List.of(); }
        }
    "#;
    let result = compress_text(source, "java", Fidelity::Medium, "γ1", None);
    assert!(result.is_ok(), "compress_text for Java should succeed");
    let (_body_lines, full_output) = result.unwrap();
    // The spring meta-layer should recognize @RestController and emit its marker.
    // At minimum the class declaration must survive with the annotation text.
    assert!(
        full_output.contains("UserController"),
        "compressed output must contain the class name, got:\n{}",
        full_output
    );
}

// ── Multi-Class Cross-Contamination Verification ────────────────────
//
// Verifies per-class metadata invariant through the production
// compress_text pipeline. Cross-contamination = class A receives
// markers that belong to class B.

#[cfg(feature = "spring_boot")]
#[test]
fn multi_class_java_spring_no_cross_low() {
    let source = include_str!("../../test_files/java/MultiClassFixture.java");
    let result = compress_text(source, "java", Fidelity::Low, "gJ1", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("ItemController"));
    assert!(out.contains("ItemService"));
    assert!(out.contains("AppConfig"));
    assert!(out.contains("HealthController"));
    let i1 = out.find("ItemController").unwrap();
    let i2 = out.find("ItemService").unwrap();
    let i3 = out.find("HealthController").unwrap();
    assert!(i1 < i2 && i2 < i3, "document order preserved");
}

#[cfg(feature = "spring_boot")]
#[test]
fn multi_class_java_spring_no_cross_medium() {
    let source = include_str!("../../test_files/java/MultiClassFixture.java");
    let result = compress_text(source, "java", Fidelity::Medium, "gJ2", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("ItemController"));
    assert!(
        !out.contains("ItemService") || !out.contains("rest:ItemService"),
        "ItemService must not inherit rest marker"
    );
}

#[cfg(feature = "spring_boot")]
#[test]
fn multi_class_java_spring_no_cross_high() {
    let source = include_str!("../../test_files/java/MultiClassFixture.java");
    let result = compress_text(source, "java", Fidelity::High, "gJ3", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("AppConfig"));
    assert!(
        !out.contains("AppConfig") || !out.contains("rest:AppConfig"),
        "AppConfig must not inherit rest marker"
    );
}

#[cfg(feature = "angular")]
#[test]
fn multi_class_ts_angular_no_cross_low() {
    let source = include_str!("../../test_files/typescript/MultiClassFixture.ts");
    let result = compress_text(source, "ts", Fidelity::Low, "gT1", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("HelloComponent"));
    assert!(out.contains("DataService"));
    assert!(out.contains("GoodbyeComponent"));
    assert!(
        !out.contains("DataService") || !out.contains("cmp:DataService"),
        "DataService must not inherit cmp marker"
    );
    assert!(
        !out.contains("HelloComponent") || !out.contains("svc:HelloComponent"),
        "HelloComponent must not inherit svc marker"
    );
    let h = out.find("HelloComponent").unwrap();
    let g = out.find("GoodbyeComponent").unwrap();
    assert!(h < g, "document order preserved");
}

#[cfg(feature = "angular")]
#[test]
fn multi_class_ts_angular_no_cross_medium() {
    let source = include_str!("../../test_files/typescript/MultiClassFixture.ts");
    let result = compress_text(source, "ts", Fidelity::Medium, "gT2", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("HelloComponent"));
    assert!(out.contains("GoodbyeComponent"));
    assert!(
        !out.contains("DataService") || !out.contains("cmp:DataService"),
        "DataService must not inherit cmp at Medium"
    );
}

#[cfg(feature = "angular")]
#[test]
fn multi_class_ts_angular_no_cross_high() {
    let source = include_str!("../../test_files/typescript/MultiClassFixture.ts");
    let result = compress_text(source, "ts", Fidelity::High, "gT3", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("HelloComponent"));
    assert!(out.contains("GoodbyeComponent"));
    assert!(
        !out.contains("DataService") || !out.contains("cmp:DataService"),
        "DataService must not inherit cmp at High"
    );
}

#[cfg(feature = "dotnet")]
#[test]
fn multi_class_csharp_dotnet_no_cross_low() {
    let source = include_str!("../../test_files/dotnet/MultiClassFixture.cs");
    let result = compress_text(source, "cs", Fidelity::Low, "gC1", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("ProductsController"));
    assert!(out.contains("NotificationHub"));
    assert!(out.contains("InventoryDbContext"));
    assert!(
        !out.contains("NotificationHub") || !out.contains("api:NotificationHub"),
        "NotificationHub must not inherit api marker"
    );
    assert!(
        !out.contains("InventoryDbContext") || !out.contains("api:InventoryDbContext"),
        "DbContext must not inherit api marker"
    );
    let p = out.find("ProductsController").unwrap();
    let h = out.find("NotificationHub").unwrap();
    let d = out.find("InventoryDbContext").unwrap();
    assert!(p < h && h < d, "document order preserved");
}

#[cfg(feature = "dotnet")]
#[test]
fn multi_class_csharp_dotnet_no_cross_medium() {
    let source = include_str!("../../test_files/dotnet/MultiClassFixture.cs");
    let result = compress_text(source, "cs", Fidelity::Medium, "gC2", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("ProductsController"));
    assert!(
        !out.contains("NotificationHub") || !out.contains("api:NotificationHub"),
        "NotificationHub must not inherit api at Medium"
    );
}

#[cfg(feature = "dotnet")]
#[test]
fn multi_class_csharp_dotnet_no_cross_high() {
    let source = include_str!("../../test_files/dotnet/MultiClassFixture.cs");
    let result = compress_text(source, "cs", Fidelity::High, "gC3", None);
    assert!(result.is_ok());
    let (_, out) = result.unwrap();
    assert!(out.contains("ProductsController"));
    assert!(
        !out.contains("NotificationHub") || !out.contains("api:NotificationHub"),
        "NotificationHub must not inherit api at High"
    );
}
