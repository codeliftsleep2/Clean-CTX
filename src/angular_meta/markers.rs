// src/angular_meta/markers.rs
//
// `Φ` (Phi) marker construction & expansion.
//
// # Mirrors `compression::markers` shape
//
// Just like the `⊕` behavior markers (`⊕guard`, `⊕loop`, `⊕⇒`, `⊕!`)
// the compression pipeline emits a marker token and the decompressor
// expands it back to a human-readable form, the `Φ` meta markers
// (`Φcmp:`, `Φsvc:`, `Φin:`, `Φout:`, …) are a parallel vocabulary
// for framework-annotation. They are:
//   - emitted by `run_meta_layer` after the compacted class entry
//   - collapsed by `expand_phi_in_line` on decompression
//
// # Marker vocabulary
//
// | Marker   | Expansion (Phase 1)                          |
// |----------|----------------------------------------------|
// | Φcmp:    | @Component                                   |
// | Φdir:    | @Directive                                   |
// | Φpipe:   | @Pipe                                        |
// | Φsvc:    | @Injectable                                  |
// | Φmod:    | @NgModule                                    |
// | Φin:     | @Input                                       |
// | Φout:    | @Output                                      |
// | Φinjects:| constructor injection (private/protected)   |
// | Φgraph:  | cross-file dependency graph (Phase 3 only)   |
// | ΦBUNDLE  | file-triplet bundle (Phase 2 only)           |
// | ΦMAP     | workspace meta-map footer (Phase 2 only)     |
//
// # Why a single-character prefix?
//
// `Φ` (U+03A6) is visually distinct from `$` (opcode), `⊕` (behavior),
// and `α/β/γ` (path). It cannot be confused for an identifier in
// TypeScript / JavaScript / C# source, so the markers are safe to
// interleave with the existing notation without escaping.

/// Build a `Φcmp:<ClassName> [sel=… tpl=… sty=…]` marker line from
/// the parsed fields of a `@Component` decorator object.
///
/// Unknown fields are silently dropped (forward-compat with future
/// Angular versions). The class name is always emitted even if no
/// other fields were parsed.
pub fn build_component_line(class_name: &str, fields: &ComponentFields) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(sel) = &fields.selector {
        parts.push(format!("sel={}", sel));
    }
    if let Some(tpl) = &fields.template_url {
        parts.push(format!("tpl={}", tpl));
    } else if let Some(tpl) = &fields.template {
        parts.push(format!("tpl={}", tpl));
    }
    if let Some(sty) = &fields.style_urls {
        if sty.len() == 1 {
            parts.push(format!("sty={}", sty[0]));
        } else if !sty.is_empty() {
            parts.push(format!("sty=[{}]", sty.join(",")));
        }
    } else if let Some(sty) = &fields.styles {
        parts.push(format!("sty={}", sty));
    }
    if parts.is_empty() {
        format!("Φcmp:{}", class_name)
    } else {
        format!("Φcmp:{} {}", class_name, parts.join(" "))
    }
}

/// Build a `Φsvc:<ClassName> [scope=…]` marker line from a parsed
/// `@Injectable` decorator.
pub fn build_service_line(class_name: &str, provided_in: Option<&str>) -> String {
    match provided_in {
        Some(scope) => format!("Φsvc:{} scope={}", class_name, scope),
        None => format!("Φsvc:{}", class_name),
    }
}

/// Build a `Φmod:<ClassName> [decl=… imp=… exp=…]` marker line from a
/// parsed `@NgModule` decorator.
pub fn build_module_line(class_name: &str, decl: &[String], imp: &[String], exp: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !decl.is_empty() {
        parts.push(format!("decl=[{}]", decl.join(",")));
    }
    if !imp.is_empty() {
        parts.push(format!("imp=[{}]", imp.join(",")));
    }
    if !exp.is_empty() {
        parts.push(format!("exp=[{}]", exp.join(",")));
    }
    if parts.is_empty() {
        format!("Φmod:{}", class_name)
    } else {
        format!("Φmod:{} {}", class_name, parts.join(" "))
    }
}

/// Build a `Φdir:<ClassName> [sel=…]` marker line from a parsed
/// `@Directive` decorator.
pub fn build_directive_line(class_name: &str, selector: Option<&str>) -> String {
    match selector {
        Some(sel) => format!("Φdir:{} sel={}", class_name, sel),
        None => format!("Φdir:{}", class_name),
    }
}

/// Build a `Φpipe:<ClassName> [name=…]` marker line from a parsed
/// `@Pipe` decorator.
pub fn build_pipe_line(class_name: &str, name: Option<&str>) -> String {
    match name {
        Some(n) => format!("Φpipe:{} name={}", class_name, n),
        None => format!("Φpipe:{}", class_name),
    }
}

/// Build a `Φin:<fieldName> [alias=…]` marker line for an `@Input()`
/// field.
pub fn build_input_line(field_name: &str, alias: Option<&str>) -> String {
    match alias {
        Some(a) => format!("Φin:{} alias={}", field_name, a),
        None => format!("Φin:{}", field_name),
    }
}

/// Build a `Φout:<fieldName> [alias=…]` marker line for an `@Output()`
/// field.
pub fn build_output_line(field_name: &str, alias: Option<&str>) -> String {
    match alias {
        Some(a) => format!("Φout:{} alias={}", field_name, a),
        None => format!("Φout:{}", field_name),
    }
}

/// Build a `Φinjects:[<Type>,…]` marker line for constructor
/// parameters with `private` / `protected` access modifiers. The
/// Phase 1 implementation emits class names only (no `α` aliases yet
/// — that is Phase 3).
pub fn build_injects_line(types: &[String]) -> String {
    format!("Φinjects:[{}]", types.join(","))
}

/// Parsed fields of a `@Component({...})` decorator object. All
/// fields are optional; missing fields are simply not emitted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentFields {
    pub selector: Option<String>,
    pub template_url: Option<String>,
    pub template: Option<String>,
    pub style_urls: Option<Vec<String>>,
    pub styles: Option<String>,
}

/// Expand every recognised `Φ…` marker in a line back to its
/// decorator form. Used by the decompressor.
///
/// This is the round-trip counterpart to the `build_*` helpers
/// above. It is **deliberately conservative**: only the marker
/// prefix is rewritten (`Φcmp:` → `@Component`); the trailing
/// `key=value` attributes are left untouched because they are
/// already human-readable in the compressed form.
///
/// Unknown `Φ` tokens (e.g. `ΦBUNDLE` from Phase 2) are passed
/// through unchanged.
pub fn expand_phi_in_line(line: &str) -> String {
    // Order matters: longer prefixes must be tried first so that
    // `Φinjects:` does not get partially matched by `Φin:`.
    // (Currently no overlap exists — `Φin:` is exactly 3 chars and
    // `Φinjects:` is 8 — but the explicit ordering is cheap
    // insurance against future vocabulary additions.)
    let pairs: &[(&str, &str)] = &[
        ("Φcmp:", "@Component "),
        ("Φdir:", "@Directive "),
        ("Φpipe:", "@Pipe "),
        ("Φsvc:", "@Injectable "),
        ("Φmod:", "@NgModule "),
        ("Φinjects:", "@Inject "),
        ("Φgraph:", "@Graph "),
        ("Φin:", "@Input "),
        ("Φout:", "@Output "),
        ("ΦBUNDLE", "@Bundle "),
        ("ΦMAP", "@Map "),
        ("Φtpl:", "@Template "),
        ("Φsty:", "@Style "),
    ];
    let mut s = line.to_string();
    for (from, to) in pairs {
        s = s.replace(from, to);
    }
    s
}

/// Expand a single `Φ` marker token to its decorator form. Returns
/// `None` for unknown markers so the caller can pass them through.
#[allow(dead_code)]
pub fn expand_phi(token: &str) -> Option<&'static str> {
    match token {
        "Φcmp" => Some("@Component"),
        "Φdir" => Some("@Directive"),
        "Φpipe" => Some("@Pipe"),
        "Φsvc" => Some("@Injectable"),
        "Φmod" => Some("@NgModule"),
        "Φin" => Some("@Input"),
        "Φout" => Some("@Output"),
        "Φinjects" => Some("@Inject"),
        "Φgraph" => Some("@Graph"),
        "ΦBUNDLE" => Some("@Bundle"),
        "ΦMAP" => Some("@Map"),
        "Φtpl" => Some("@Template"),
        "Φsty" => Some("@Style"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/angular_meta/markers.rs"]
mod tests;
