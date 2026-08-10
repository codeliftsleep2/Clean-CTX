// src/angular_meta/rx.rs
//
// RxJS Meta-Layer — Phase 1 of the Angular Ecosystem Deepening.
//
// Detects and compresses RxJS observable declarations, subject
// instantiations, pipe chains, and static combinators in Angular
// TypeScript files. Outputs a `// --- Φ RxJS Meta ---` block.
//
// # Purely additive
//
// The RxJS meta-layer never modifies existing TS compression output.
// It only appends a `Φ RxJS Meta` block below the existing compacted
// class. Non-RxJS files pay zero overhead (import-gate detection).
//
// # Marker architecture
//
// This module defines its own `RxJsKind` sub-enum (not added to the
// existing `PhiLineKind` in `markers.rs`) to avoid a 41-variant
// monolith. The `expand_phi_in_line` function is chained into the
// existing Angular expansion via `markers.rs`.

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;

// ---------------------------------------------------------------------------
// RxJsKind — single source of truth for RxJS marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind for RxJS constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RxJsKind {
    Observable,
    Subject,
    PipeRx,
    Map,
    Tap,
    Filter,
    Catch,
    Finalize,
    Delay,
    Combine,
    Share,
    To,
    With,
    Scan,
    Distinct,
    Retry,
}

impl PhiMarker for RxJsKind {
    /// The `Φ` marker prefix for this kind (e.g. `"Φobs:"`).
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Observable => "Φobs:",
            Self::Subject => "Φsubject:",
            Self::PipeRx => "ΦpipeRx:",
            Self::Map => "Φmap:",
            Self::Tap => "Φtap:",
            Self::Filter => "Φfilter:",
            Self::Catch => "Φcatch:",
            Self::Finalize => "Φfinalize:",
            Self::Delay => "Φdelay:",
            Self::Combine => "Φcombine:",
            Self::Share => "Φshare:",
            Self::To => "Φto:",
            Self::With => "Φwith:",
            Self::Scan => "Φscan:",
            Self::Distinct => "Φdistinct:",
            Self::Retry => "Φretry:",
        }
    }

    /// The human-readable expansion (e.g. `"Observable"`).
    /// Does NOT include the trailing space.
    fn expansion(self) -> &'static str {
        match self {
            Self::Observable => "Observable",
            Self::Subject => "Subject",
            Self::PipeRx => "PipeRx",
            Self::Map => "Map",
            Self::Tap => "Tap",
            Self::Filter => "Filter",
            Self::Catch => "CatchError",
            Self::Finalize => "Finalize",
            Self::Delay => "Delay",
            Self::Combine => "CombineLatest",
            Self::Share => "Share",
            Self::To => "FirstValueFrom",
            Self::With => "WithLatestFrom",
            Self::Scan => "Scan",
            Self::Distinct => "DistinctUntilChanged",
            Self::Retry => "Retry",
        }
    }

    /// All variants in a canonical order (longer prefixes first to
    /// prevent partial-match issues in string replacement).
    fn all_in_expand_order() -> &'static [RxJsKind] {
        &[
            Self::Observable,  // Φobs:      (5 chars)
            Self::Subject,     // Φsubject:  (9 chars)
            Self::PipeRx,      // ΦpipeRx:   (8 chars)
            Self::Map,         // Φmap:      (5 chars)
            Self::Tap,         // Φtap:      (5 chars)
            Self::Filter,      // Φfilter:   (8 chars)
            Self::Catch,       // Φcatch:    (7 chars)
            Self::Finalize,    // Φfinalize: (10 chars)
            Self::Delay,       // Φdelay:    (7 chars)
            Self::Combine,     // Φcombine:  (9 chars)
            Self::Share,       // Φshare:    (7 chars)
            Self::To,          // Φto:       (4 chars)
            Self::With,        // Φwith:     (6 chars)
            Self::Scan,        // Φscan:     (6 chars)
            Self::Distinct,    // Φdistinct: (10 chars)
            Self::Retry,       // Φretry:    (7 chars)
        ]
    }

    /// Look up an [`RxJsKind`] by its marker token string (without
    /// the trailing colon). Returns `None` for unknown tokens.
    fn from_token(token: &str) -> Option<RxJsKind> {
        match token {
            "Φobs" => Some(Self::Observable),
            "Φsubject" => Some(Self::Subject),
            "ΦpipeRx" => Some(Self::PipeRx),
            "Φmap" => Some(Self::Map),
            "Φtap" => Some(Self::Tap),
            "Φfilter" => Some(Self::Filter),
            "Φcatch" => Some(Self::Catch),
            "Φfinalize" => Some(Self::Finalize),
            "Φdelay" => Some(Self::Delay),
            "Φcombine" => Some(Self::Combine),
            "Φshare" => Some(Self::Share),
            "Φto" => Some(Self::To),
            "Φwith" => Some(Self::With),
            "Φscan" => Some(Self::Scan),
            "Φdistinct" => Some(Self::Distinct),
            "Φretry" => Some(Self::Retry),
            _ => None,
        }
    }

    /// Returns the token string (without trailing `:`) for a given kind.
    fn token(self) -> &'static str {
        match self {
            Self::Observable => "Φobs",
            Self::Subject => "Φsubject",
            Self::PipeRx => "ΦpipeRx",
            Self::Map => "Φmap",
            Self::Tap => "Φtap",
            Self::Filter => "Φfilter",
            Self::Catch => "Φcatch",
            Self::Finalize => "Φfinalize",
            Self::Delay => "Φdelay",
            Self::Combine => "Φcombine",
            Self::Share => "Φshare",
            Self::To => "Φto",
            Self::With => "Φwith",
            Self::Scan => "Φscan",
            Self::Distinct => "Φdistinct",
            Self::Retry => "Φretry",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// The kind of RxJS subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    Subject,
    BehaviorSubject,
    ReplaySubject,
    AsyncSubject,
}

impl std::fmt::Display for SubjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subject => write!(f, "Subject"),
            Self::BehaviorSubject => write!(f, "BehaviorSubject"),
            Self::ReplaySubject => write!(f, "ReplaySubject"),
            Self::AsyncSubject => write!(f, "AsyncSubject"),
        }
    }
}

/// A single observable field declaration.
#[derive(Debug, Clone)]
pub struct ObservableDecl {
    pub name: String,
    pub source: Option<String>,   // "http.get", "of", "from", etc.
    pub type_param: Option<String>,
}

/// A single subject instantiation.
#[derive(Debug, Clone)]
pub struct SubjectDecl {
    pub name: String,
    pub kind: SubjectKind,
    pub initial_value: Option<String>,
    pub type_param: Option<String>,
}

/// A single pipe operator in a chain.
#[derive(Debug, Clone)]
pub struct PipeOperator {
    pub kind: RxJsKind,
    pub operator_name: String,
    pub arg_summary: Option<String>,  // High fidelity only
}

/// A pipe chain attached to an observable field.
#[derive(Debug, Clone)]
pub struct PipeChain {
    pub owner: String,
    pub operators: Vec<PipeOperator>,
}

/// A static combinator declaration (combineLatest, forkJoin, etc.).
#[derive(Debug, Clone)]
pub struct CombinatorDecl {
    pub kind: RxJsKind,
    pub name: String,
    pub args: Vec<String>,
}

/// The complete RxJS shape extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct RxShape {
    pub observables: Vec<ObservableDecl>,
    pub subjects: Vec<SubjectDecl>,
    pub pipes: Vec<PipeChain>,
    pub combinators: Vec<CombinatorDecl>,
}

impl RxShape {
    /// Returns `true` if there are no RxJS artifacts to emit.
    pub fn is_empty(&self) -> bool {
        self.observables.is_empty()
            && self.subjects.is_empty()
            && self.pipes.is_empty()
            && self.combinators.is_empty()
    }

    /// Render the full `Φ RxJS Meta` block at the given fidelity.
    pub fn render(&self, fidelity: Fidelity) -> String {
        self.render_with_config(fidelity, None)
    }

    /// Render the full `Φ RxJS Meta` block at the given fidelity,
    /// honoring the `min_pipe_operators` config (when provided).
    ///
    /// `min_pipe_operators` suppresses `ΦpipeRx:` blocks with fewer
    /// than N operators (default 2) to prevent noise from trivial
    /// single-operator chains.
    pub fn render_with_config(&self, fidelity: Fidelity, min_pipe_operators: Option<usize>) -> String {
        if self.is_empty() {
            return String::new();
        }

        let min_ops = min_pipe_operators.unwrap_or(2);

        let mut s = String::new();
        s.push_str("// --- Φ RxJS Meta ---\n");

        // Observable fields (all fidelities)
        for obs in &self.observables {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φobs:{}\n", obs.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    if let Some(ref src) = obs.source {
                        s.push_str(&format!("  Φobs:{} → {}\n", obs.name, src));
                    } else {
                        s.push_str(&format!("  Φobs:{}\n", obs.name));
                    }
                }
            }
        }

        // Subject declarations (all fidelities)
        for subj in &self.subjects {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φsubject:{}\n", subj.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    if let Some(ref iv) = subj.initial_value {
                        s.push_str(&format!("  Φsubject:{} = new {}({})\n",
                            subj.name, subj.kind, iv));
                    } else {
                        s.push_str(&format!("  Φsubject:{} = new {}<{}>()\n",
                            subj.name, subj.kind,
                            subj.type_param.as_deref().unwrap_or("?")));
                    }
                }
            }
        }

        // Pipe chains (suppressed when below min_pipe_operators).
        for pipe in &self.pipes {
            if pipe.operators.len() < min_ops {
                continue;
            }
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  ΦpipeRx:{}\n", pipe.owner));
                }
                Fidelity::Medium => {
                    let ops: Vec<&str> = pipe.operators.iter()
                        .map(|op| op.operator_name.as_str())
                        .collect();
                    s.push_str(&format!("  ΦpipeRx:{} = {}\n", pipe.owner, ops.join(" → ")));
                }
                Fidelity::High => {
                    let mut pipe_line = format!("  ΦpipeRx:{} = pipe(\n", pipe.owner);
                    for (i, op) in pipe.operators.iter().enumerate() {
                        let comma = if i < pipe.operators.len() - 1 { "," } else { "" };
                        if let Some(ref arg) = op.arg_summary {
                            pipe_line.push_str(&format!("    {}:{}{}\n", op.operator_name, arg, comma));
                        } else {
                            pipe_line.push_str(&format!("    {}{}\n", op.operator_name, comma));
                        }
                    }
                    pipe_line.push_str("  )");
                    s.push_str(&pipe_line);
                    s.push('\n');
                }
            }
        }

        // Static combinators
        for comb in &self.combinators {
            s.push_str(&format!("  Φcombine:{} {}\n", comb.name, comb.args.join(", ")));
        }

        s
    }
}

// ---------------------------------------------------------------------------
// Detection — import gate
// ---------------------------------------------------------------------------

/// Check whether the source file has RxJS imports.
/// Returns true if the file imports from `rxjs` or `rxjs/operators`.
pub fn has_rxjs_imports(source: &str) -> bool {
    // Simple string-based import scan (consistent with detect.rs approach).
    // We check for the most common RxJS import patterns.
    source.contains("from 'rxjs'")
        || source.contains("from \"rxjs\"")
        || source.contains("from 'rxjs/operators'")
        || source.contains("from \"rxjs/operators\"")
        || source.contains("from 'rxjs/internal'")
        || source.contains("from \"rxjs/internal\"")
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Extract the RxJS shape from a source file.
///
/// Returns `None` when the file has no RxJS imports (zero overhead).
/// Returns `Some(RxShape)` with detected observables, subjects, pipes,
/// and combinators.
pub fn extract_rx_shape(source: &str, _fidelity: Fidelity) -> Option<RxShape> {
    // Import gate: skip non-RxJS files
    if !has_rxjs_imports(source) {
        return None;
    }

    let mut shape = RxShape::default();

    // Extract observable field declarations
    extract_observables(source, &mut shape);

    // Extract subject instantiations
    extract_subjects(source, &mut shape);

    // Extract pipe chains
    extract_pipe_chains(source, &mut shape);

    // Extract static combinator calls
    extract_combinators(source, &mut shape);

    if shape.is_empty() {
        return None;
    }

    Some(shape)
}

/// Extract observable field declarations from the source.
///
/// Detects:
/// - `Observable<T>` type annotations
/// - `$`-suffixed field names
/// - Creation functions: `of(...)`, `from(...)`, `interval(...)`,
///   `timer(...)`, `fromEvent(...)`
/// - Service calls: `http.get(...)`, `this.http.get(...)`
fn extract_observables(source: &str, shape: &mut RxShape) {
    // Scan line-by-line for observable patterns
    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments and strings
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }

        // Pattern: `name$: Observable<T>` or `name$ = http.get(...)`
        // or `name$ = of(...)`, `name$ = from(...)`, etc.
        if let Some(obs) = extract_observable_from_line(trimmed) {
            shape.observables.push(obs);
        }
    }
}

/// Try to extract an observable declaration from a single line.
fn extract_observable_from_line(line: &str) -> Option<ObservableDecl> {
    // Check for service calls first (e.g. `name$: Observable<T> = this.http.get(...)`)
    // so the source is captured when the type annotation is also present.
    if let Some(obs) = extract_service_call_observable(line) {
        return Some(obs);
    }

    // Check for `Observable<T>` type annotation
    // Pattern: `name$: Observable<Type>` or `name: Observable<Type>`
    if let Some(idx) = line.find(": Observable<") {
        let before = &line[..idx];
        let name = extract_field_name(before)?;
        let rest = &line[idx + ": Observable<".len()..];
        let type_param = rest.split('>').next().map(|s| s.trim().to_string());
        return Some(ObservableDecl {
            name,
            source: None,
            type_param,
        });
    }

    // Check for `Observable<Type>` without field prefix (e.g. return type)
    if line.starts_with("Observable<") {
        // Not a field declaration — skip
        return None;
    }

    // Check for creation functions: `name$ = of(...)`, `name$ = from(...)`, etc.
    let creation_funcs = [
        "of(", "from(", "interval(", "timer(", "fromEvent(",
        "ajax(", "defer(", "empty(", "never(", "throwError(",
        "iif(", "merge(", "concat(", "race(", "zip(",
    ];

    for func in &creation_funcs {
        if let Some(idx) = line.find(&format!(" = {}", func)) {
            let before = &line[..idx];
            let name = extract_field_name(before)?;
            return Some(ObservableDecl {
                name,
                source: Some(func.trim_end_matches('(').to_string()),
                type_param: None,
            });
        }
    }

    None
}

/// Extract an observable from a service-call assignment pattern.
/// Handles lines like `name$: Observable<T> = this.http.get(...)` or
/// `name$ = this.http.get(...)`.
fn extract_service_call_observable(line: &str) -> Option<ObservableDecl> {
    // Check for service calls: `name$ = this.http.get(...)` or `name$ = http.get(...)`
    // or `name$: Observable<T> = this.http.get(...)`
    let eq_idx = line.find(" = ")?;
    let before = &line[..eq_idx];
    let name = extract_field_name(before)?;

    // Check various HTTP/service patterns.
    // Note: the call may include a generic type param, e.g.
    // `http.get<User[]>(...)` — so we match `http.get` followed by
    // either `(` or `<`.
    let after = &line[eq_idx + 3..];
    let service_patterns = [
        "http.get", "http.post", "http.put", "http.delete", "http.patch",
        "this.http.get", "this.http.post", "this.http.put",
        "this.http.delete", "this.http.patch",
    ];

    for pat in &service_patterns {
        if after.starts_with(pat) || after.contains(pat) {
            // Extract the URL or first argument.
            // The pattern may be followed by `<Type>` (generic) then `(`.
            let rest = after.split(pat).nth(1).unwrap_or("");
            // Skip generic type params: `<...>`
            let rest = if let Some(gt_idx) = rest.find('>') {
                &rest[gt_idx + 1..]
            } else {
                rest
            };
            // Now find the first `(` and extract the URL argument.
            let url = if let Some(open_idx) = rest.find('(') {
                let after_open = &rest[open_idx + 1..];
                after_open.split(')').next().unwrap_or("").trim().to_string()
            } else {
                String::new()
            };
            return Some(ObservableDecl {
                name,
                source: Some(format!("http.get({})", url)),
                type_param: None,
            });
        }
    }

    // Check for `this.service.method()` patterns
    if after.contains(".get(") || after.contains(".post(") || after.contains(".put(")
        || after.contains(".delete(") || after.contains(".patch(") {
        // Extract up to the first '('
        let method_call = after.split('(').next().unwrap_or("").trim();
        return Some(ObservableDecl {
            name,
            source: Some(method_call.to_string()),
            type_param: None,
        });
    }

    None
}

/// Extract field name from before a type annotation or assignment.
/// Handles: `name$`, `private name$`, `readonly name$`, `public name$`,
/// `protected name$`, `static name$`.
fn extract_field_name(before: &str) -> Option<String> {
    let trimmed = before.trim();
    // Split by whitespace and take the last word (the field name)
    let name = trimmed.split_whitespace().last()?;
    if name.is_empty() {
        return None;
    }
    // Filter out keywords and type names
    if name == "private" || name == "public" || name == "protected"
        || name == "readonly" || name == "static" || name == "const"
        || name == "let" || name == "var"
    {
        return None;
    }
    Some(name.to_string())
}

/// Extract subject instantiations from the source.
///
/// Detects:
/// - `new Subject<T>()`
/// - `new BehaviorSubject<T>(initialValue)`
/// - `new ReplaySubject<T>(n)`
/// - `new AsyncSubject<T>()`
fn extract_subjects(source: &str, shape: &mut RxShape) {
    let subject_patterns = [
        ("new Subject<", SubjectKind::Subject),
        ("new BehaviorSubject<", SubjectKind::BehaviorSubject),
        ("new ReplaySubject<", SubjectKind::ReplaySubject),
        ("new AsyncSubject<", SubjectKind::AsyncSubject),
    ];

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }

        for (pattern, kind) in &subject_patterns {
            if let Some(idx) = trimmed.find(pattern) {
                // Extract field name before the `new` keyword.
                // The line looks like: `name = new Subject<T>()` or
                // `private name = new Subject<T>()`.
                let before = &trimmed[..idx].trim();
                // Take the last non-empty token, stripping trailing `=`.
                let name = before.split_whitespace()
                    .last()
                    .map(|s| s.trim_end_matches('=').trim().to_string())
                    .filter(|s| !s.is_empty() && *s != "=" && *s != ":" && *s != "private"
                        && *s != "public" && *s != "protected" && *s != "readonly"
                        && *s != "static" && *s != "const" && *s != "let" && *s != "var"
                        && *s != "new");
                // If the last token was `=` (e.g. `name = new Subject`),
                // the actual field name is the second-to-last token.
                let name = name.or_else(|| {
                    let tokens: Vec<&str> = before.split_whitespace().collect();
                    if tokens.len() >= 2 {
                        let candidate = tokens[tokens.len() - 2].trim_end_matches('=').trim();
                        if !candidate.is_empty() && candidate != "=" && candidate != "new"
                            && candidate != "private" && candidate != "public"
                            && candidate != "protected" && candidate != "readonly"
                            && candidate != "static" && candidate != "const"
                            && candidate != "let" && candidate != "var"
                        {
                            Some(candidate.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                let rest = &trimmed[idx + pattern.len()..];

                // Extract type parameter
                let type_param = rest.split('>').next().map(|s| s.trim().to_string());

                // Extract initial value (for BehaviorSubject) or buffer size (for ReplaySubject)
                let rest_after_type = rest.split('>').nth(1).unwrap_or("");
                let initial_value = if rest_after_type.starts_with('(') {
                    // Extract the constructor argument
                    let args = rest_after_type.trim_start_matches('(');
                    let arg = args.split(')').next().map(|s| s.trim().to_string());
                    arg.filter(|s| !s.is_empty() && *s != ";" && *s != ",")
                } else {
                    None
                };

                shape.subjects.push(SubjectDecl {
                    name: name.unwrap_or_else(|| "?".to_string()),
                    kind: *kind,
                    initial_value,
                    type_param,
                });
                break;
            }
        }
    }
}

/// Extract pipe chains from the source.
///
/// Detects `.pipe(operator1(...), operator2(...), ...)` chains.
/// The pipe must be assigned to a field or variable.
fn extract_pipe_chains(source: &str, shape: &mut RxShape) {
    // This is a simplified extraction — we look for `.pipe(` patterns
    // and extract the operator sequence.
    // Full AST-based extraction would be more accurate, but string-based
    // is consistent with the existing Angular meta-layer approach.

    // We need to handle multi-line pipe bodies. The strategy:
    // 1. Find each `.pipe(` occurrence.
    // 2. Collect the full pipe body (which may span multiple lines)
    //    by tracking bracket depth.
    // 3. Extract operators from the collected body.

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            i += 1;
            continue;
        }

        // Check for `.pipe(` pattern
        if let Some(pipe_idx) = trimmed.find(".pipe(") {
            // Extract the owner (the expression before .pipe).
            // For `users$ = this.refreshTrigger.pipe(`, the owner is
            // `users$` (the field name before `=`). For
            // `this.refreshTrigger.pipe(`, the owner is the expression
            // itself.
            let before = &trimmed[..pipe_idx].trim();
            let owner = if let Some(eq_idx) = before.find('=') {
                // The assignment LHS is everything before `=`. Strip
                // access modifiers (`private`, `public`, etc.) and type
                // annotations (`name: Observable<T>` → `name`) to get the
                // bare field name. The type annotation must be stripped
                // FIRST (split on ':'), then modifiers, then the last
                // whitespace token is the field name.
                let lhs = before[..eq_idx].trim();
                // Strip the type annotation: `private users$: Observable<User[]>`
                // → `private users$`.
                let name_part = lhs.split(':').next().unwrap_or(lhs).trim();
                let mut words: Vec<&str> = name_part.split_whitespace().collect();
                // Drop a leading access modifier / `readonly` / `static`.
                while let Some(first) = words.first() {
                    if matches!(*first, "private" | "public" | "protected" | "readonly" | "static") {
                        words.remove(0);
                    } else {
                        break;
                    }
                }
                // The last remaining token is the field name.
                let candidate = words.last().map(|s| s.to_string()).unwrap_or_default();
                candidate
                    .split(':')
                    .next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && *s != "=" && *s != ":")
                    .unwrap_or_else(|| "?".to_string())
            } else {
                before.split_whitespace()
                    .last()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && *s != "return" && *s != ")" && *s != "}")
                    .unwrap_or_else(|| "?".to_string())
            };

            // Collect the full pipe body, which may span multiple lines.
            // After `.pipe(`, we're at depth 1 (the pipe's own paren).
            // We need to find the `)` that brings us back to depth 0.
            let mut pipe_body = String::new();
            let mut depth = 1; // the `.pipe(` itself
            let mut j = i;
            let mut found_close = false;

            // Compute the offset between the trimmed and untrimmed line
            // so `pipe_idx` (computed on the trimmed line) maps correctly
            // to the untrimmed line.
            let leading_ws = lines[i].len() - lines[i].trim_start().len();

            while j < lines.len() {
                let line = lines[j];
                let start_idx = if j == i {
                    leading_ws + pipe_idx + ".pipe(".len()
                } else {
                    0
                };
                let segment = &line[start_idx..];

                for ch in segment.chars() {
                    match ch {
                        '(' => { depth += 1; }
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                // This closes the pipe() call.
                                found_close = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                    if found_close { break; }
                }

                if j == i {
                    // Include the rest of the first line after `.pipe(`
                    pipe_body.push_str(segment);
                } else {
                    pipe_body.push('\n');
                    pipe_body.push_str(line);
                }

                if found_close { break; }
                j += 1;
            }

            // Extract operators from the collected body
            let operators = extract_operators(&pipe_body);

            if !operators.is_empty() {
                // The owner (e.g. `users$`) is also an observable field
                // declaration — register it so Low fidelity emits `Φobs:`.
                if owner != "?" && !shape.observables.iter().any(|o| o.name == owner) {
                    shape.observables.push(ObservableDecl {
                        name: owner.clone(),
                        source: None,
                        type_param: None,
                    });
                }
                shape.pipes.push(PipeChain {
                    owner,
                    operators,
                });
            }

            // Skip to the line after the pipe body
            i = j + 1;
            continue;
        }

        i += 1;
    }
}

/// Extract operators from inside a `.pipe(...)` call.
fn extract_operators(pipe_body: &str) -> Vec<PipeOperator> {
    let mut operators = Vec::new();
    let mut depth = 0;
    let mut current_op = String::new();
    let mut paren_depth = 0;

    for ch in pipe_body.chars() {
        match ch {
            '(' => {
                if paren_depth > 0 {
                    // Nested paren inside an operator's arguments
                    depth += 1;
                    current_op.push(ch);
                } else {
                    // Start of a new operator
                    paren_depth += 1;
                    current_op.push(ch);
                }
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                    current_op.push(ch);
                } else if paren_depth > 0 {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        // End of this operator — don't push the closing
                        // paren; it's the operator's terminator.
                        if let Some(op) = parse_operator(&current_op) {
                            operators.push(op);
                        }
                        current_op.clear();
                    } else {
                        current_op.push(ch);
                    }
                }
            }
            ',' => {
                if paren_depth == 0 && depth == 0 {
                    // Separator between operators — flush current
                    if !current_op.trim().is_empty() {
                        if let Some(op) = parse_operator(&current_op) {
                            operators.push(op);
                        }
                        current_op.clear();
                    }
                } else {
                    current_op.push(ch);
                }
            }
            ' ' | '\t' | '\n' | '\r' => {
                if paren_depth > 0 || depth > 0 {
                    current_op.push(ch);
                }
                // Skip whitespace outside parens
            }
            _ => {
                current_op.push(ch);
            }
        }
    }

    // Flush remaining operator
    if !current_op.trim().is_empty() {
        if let Some(op) = parse_operator(&current_op) {
            operators.push(op);
        }
    }

    operators
}

/// Parse a single operator expression (e.g. `switchMap(x => x.getUsers())`).
fn parse_operator(op_text: &str) -> Option<PipeOperator> {
    let trimmed = op_text.trim().trim_end_matches(')').trim();
    if trimmed.is_empty() {
        return None;
    }

    // The operator name is the first word before '('
    let paren_idx = trimmed.find('(')?;
    let name = &trimmed[..paren_idx];
    let args = &trimmed[paren_idx + 1..];

    if name.is_empty() {
        return None;
    }

    // Determine the RxJsKind from the operator name
    let kind = operator_to_kind(name);

    // Build arg summary (High fidelity only — we store it but
    // the caller decides fidelity)
    let arg_summary = if args.is_empty() {
        None
    } else {
        // Truncate long arguments
        let summary = if args.len() > 40 {
            format!("{}...", &args[..37])
        } else {
            args.to_string()
        };
        Some(summary)
    };

    Some(PipeOperator {
        kind,
        operator_name: name.to_string(),
        arg_summary,
    })
}

/// Map an operator name to its RxJsKind.
fn operator_to_kind(name: &str) -> RxJsKind {
    match name {
        "map" | "mergeMap" | "switchMap" | "concatMap" | "exhaustMap" => RxJsKind::Map,
        "tap" | "do" => RxJsKind::Tap,
        "filter" => RxJsKind::Filter,
        "catchError" | "catch" => RxJsKind::Catch,
        "finalize" | "finally" => RxJsKind::Finalize,
        "delay" | "debounceTime" | "throttleTime" | "sampleTime" | "auditTime" => RxJsKind::Delay,
        "combineLatest" | "forkJoin" | "zip" | "race" | "merge" | "concat" => RxJsKind::Combine,
        "share" | "shareReplay" | "publish" | "publishReplay" | "multicast" => RxJsKind::Share,
        "firstValueFrom" | "lastValueFrom" | "toPromise" => RxJsKind::To,
        "withLatestFrom" => RxJsKind::With,
        "scan" | "reduce" => RxJsKind::Scan,
        "distinctUntilChanged" | "distinct" | "distinctUntilKeyChanged" => RxJsKind::Distinct,
        "retry" | "retryWhen" => RxJsKind::Retry,
        _ => RxJsKind::PipeRx, // fallback — generic pipe operator
    }
}

/// Extract static combinator calls from the source.
///
/// Detects top-level calls to:
/// - `combineLatest([a$, b$])`
/// - `forkJoin([a$, b$])`
/// - `merge(a$, b$)`
/// - `zip(a$, b$)`
/// - `race(a$, b$)`
fn extract_combinators(source: &str, shape: &mut RxShape) {
    let combinator_names = [
        "combineLatest", "forkJoin", "merge", "zip", "race",
    ];

    // Multi-line aware: scan the whole source for `name(` and collect
    // the full call body (which may span multiple lines) by tracking
    // bracket depth.
    for name in &combinator_names {
        let pattern = format!("{}(", name);
        let mut search_from = 0;
        while let Some(idx) = source[search_from..].find(&pattern) {
            let abs_idx = search_from + idx;

            // Skip matches inside comment lines (`// ...` or `* ...`).
            let line_start = source[..abs_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line = &source[line_start..abs_idx];
            let line_trim = line.trim_start();
            if line_trim.starts_with("//") || line_trim.starts_with('*') {
                search_from = abs_idx + pattern.len();
                continue;
            }

            // Collect the full call body (up to matching close paren).
            let after_start = abs_idx + pattern.len();
            let (body, _) = crate::angular_meta::util::collect_call_body(&source[after_start..]);

            // Extract arguments from the body.
            let args: Vec<String> = body.split(',')
                .map(|s| s.trim().trim_end_matches(')').trim().to_string())
                .filter(|s| !s.is_empty() && *s != "[" && *s != "]")
                .collect();

            shape.combinators.push(CombinatorDecl {
                kind: RxJsKind::Combine,
                name: name.to_string(),
                args,
            });

            search_from = after_start + body.len();
        }
    }
}

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Expand every recognised RxJS `Φ` marker in a line back to its
/// human-readable form. Used by the decompressor.
///
/// This is chained into the existing Angular `expand_phi_in_line` in
/// `markers.rs` via the [`PHI_EXPANDERS`](crate::angular_meta::phi::PHI_EXPANDERS)
/// registry.
pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<RxJsKind>(line)
}

/// Expand a single RxJS `Φ` marker token to its human-readable form.
/// Returns `None` for unknown markers.
pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<RxJsKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/rx.rs"]
mod tests;