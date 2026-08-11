// src/angular_meta/signals.rs
//
// Signals Meta-Layer — Phase 3 of the Angular Ecosystem Deepening.
//
// Detects and compresses Angular Signals constructs — `signal()`,
// `computed()`, `effect()`, `toSignal()`, `toObservable()`,
// `linkedSignal()` — in Angular TypeScript files.
//
// # Purely additive
//
// The Signals meta-layer never modifies existing TS compression output.
// It only appends a `// --- Φ Signals Meta ---` block below the existing
// compacted class. Non-Signals files pay zero overhead (import-gate
// detection via `@angular/core`).

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;

// ---------------------------------------------------------------------------
// SignalKind — single source of truth for Signals marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind for Angular Signals constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Signal,
    Computed,
    SignalEffect,
    ToSignal,
    ToObservable,
    LinkedSignal,
}

impl PhiMarker for SignalKind {
    /// The `Φ` marker prefix for this kind.
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Signal => "Φsignal:",
            Self::Computed => "Φcomputed:",
            Self::SignalEffect => "Φsig-effect:",
            Self::ToSignal => "ΦtoSignal:",
            Self::ToObservable => "ΦtoObservable:",
            Self::LinkedSignal => "ΦlinkedSignal:",
        }
    }

    /// The human-readable expansion.
    fn expansion(self) -> &'static str {
        match self {
            Self::Signal => "signal()",
            Self::Computed => "computed()",
            Self::SignalEffect => "effect()",
            Self::ToSignal => "toSignal()",
            Self::ToObservable => "toObservable()",
            Self::LinkedSignal => "linkedSignal()",
        }
    }

    /// All variants in a canonical order.
    fn all_in_expand_order() -> &'static [SignalKind] {
        &[
            Self::SignalEffect,   // Φsig-effect: (12 chars)
            Self::ToObservable,   // ΦtoObservable: (14 chars)
            Self::LinkedSignal,   // ΦlinkedSignal: (14 chars)
            Self::ToSignal,       // ΦtoSignal: (10 chars)
            Self::Computed,       // Φcomputed: (10 chars)
            Self::Signal,         // Φsignal: (8 chars)
        ]
    }

    /// Look up a [`SignalKind`] by its marker token string.
    fn from_token(token: &str) -> Option<SignalKind> {
        match token {
            "Φsignal" => Some(Self::Signal),
            "Φcomputed" => Some(Self::Computed),
            "Φsig-effect" => Some(Self::SignalEffect),
            "ΦtoSignal" => Some(Self::ToSignal),
            "ΦtoObservable" => Some(Self::ToObservable),
            "ΦlinkedSignal" => Some(Self::LinkedSignal),
            _ => None,
        }
    }

    /// Returns the token string.
    fn token(self) -> &'static str {
        match self {
            Self::Signal => "Φsignal",
            Self::Computed => "Φcomputed",
            Self::SignalEffect => "Φsig-effect",
            Self::ToSignal => "ΦtoSignal",
            Self::ToObservable => "ΦtoObservable",
            Self::LinkedSignal => "ΦlinkedSignal",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single signal declaration.
#[derive(Debug, Clone)]
pub struct SignalDecl {
    pub name: String,
    pub kind: SignalKind,
    pub type_param: Option<String>,
}

/// The complete Signals shape extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct SignalShape {
    pub signals: Vec<SignalDecl>,
}

impl SignalShape {
    /// Returns `true` if there are no signal artifacts to emit.
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    /// Render the full `Φ Signals Meta` block at the given fidelity.
    pub fn render(&self, fidelity: Fidelity) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut s = String::new();
        s.push_str("// --- Φ Signals Meta ---\n");
        for sig in &self.signals {
            match fidelity {
                Fidelity::Low => {
                    // `marker_prefix()` already includes the trailing `:`
                    // (e.g. `"Φsignal:"`), so we must NOT add another one.
                    s.push_str(&format!("  {}{}\n", sig.kind.marker_prefix(), sig.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    if let Some(ref tp) = sig.type_param {
                        // `expansion()` returns e.g. `"signal()"`.
                        // Insert the generic param before the `()` so we
                        // render `signal<number>()`, not `signal()number`.
                        let expansion = sig.kind.expansion();
                        let (base, suffix) = if let Some(paren_idx) = expansion.find('(') {
                            (&expansion[..paren_idx], &expansion[paren_idx..])
                        } else {
                            (expansion, "")
                        };
                        s.push_str(&format!("  {}{} = {}<{}>{}\n",
                            sig.kind.marker_prefix(),
                            sig.name, base, tp, suffix));
                    } else {
                        s.push_str(&format!("  {}{}\n",
                            sig.kind.marker_prefix(), sig.name));
                    }
                }
            }
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Detection — import gate
// ---------------------------------------------------------------------------

/// Check whether the source file uses Angular Signals.
/// Returns true if the file imports from `@angular/core` and uses
/// signal-related functions.
pub fn has_signal_imports(source: &str) -> bool {
    // Must import from @angular/core (signals are exported from there)
    source.contains("@angular/core")
        // And must use at least one signal function OR import a signal
        // function by name (e.g. `import { signal } from '@angular/core'`).
        && (source.contains("signal(")
            || source.contains("computed(")
            || source.contains("effect(")
            || source.contains("toSignal(")
            || source.contains("toObservable(")
            || source.contains("linkedSignal(")
            || source.contains("import { signal")
            || source.contains("import { computed")
            || source.contains("import { effect")
            || source.contains("import { toSignal")
            || source.contains("import { toObservable")
            || source.contains("import { linkedSignal"))
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Extract the Signals shape from a source file.
pub fn extract_signal_shape(source: &str, _fidelity: Fidelity) -> Option<SignalShape> {
    if !has_signal_imports(source) {
        return None;
    }
    let mut shape = SignalShape::default();

    // Extract `signal()` declarations. Note: the pattern must match
    // `= signal(` AND `= signal<T>(` (type-parameterized).
    extract_signal_decls(source, &mut shape, "= signal", SignalKind::Signal);
    // Extract `computed()` declarations
    extract_signal_decls(source, &mut shape, "= computed", SignalKind::Computed);
    // Extract `effect()` registrations. IMPORTANT: must NOT match
    // `createEffect(` (NgRx) — the pattern `effect(` would otherwise
    // match inside `createEffect(`, producing garbage `Φsig-effect:`
    // markers in every NgRx effects file. We scan for `effect(` but
    // skip occurrences immediately preceded by `create`.
    extract_effect_decls(source, &mut shape);
    // Extract `toSignal()` declarations
    extract_signal_decls(source, &mut shape, "= toSignal", SignalKind::ToSignal);
    // Extract `toObservable()` declarations
    extract_signal_decls(source, &mut shape, "= toObservable", SignalKind::ToObservable);
    // Extract `linkedSignal()` declarations
    extract_signal_decls(source, &mut shape, "= linkedSignal", SignalKind::LinkedSignal);

    if shape.is_empty() {
        return None;
    }
    Some(shape)
}

/// Extract the declaration name from the text preceding a signal call.
///
/// Returns `None` when the call is a bare statement (e.g. `effect()` in a
/// constructor body) rather than an assignment (`name = ...`). In the
/// bare-statement case the last whitespace token before the call is a
/// punctuation character (`{`, `(`, `;`, etc.) which must NOT be treated
/// as a name — the marker renders `?` instead.
fn extract_decl_name(before: &str) -> Option<String> {
    // Walk tokens from the end, skipping a trailing `=` (assignment).
    // `name = effect(` → the last token is `=`; the actual name is the
    // token before it. A bare `effect()` call in a constructor body has
    // no assignable name — the last token is punctuation (`{`, `(`, `;`)
    // which fails the identifier check and yields `?`.
    //
    // Member-expression LHS (`this.logEffect = effect(`) is normalized
    // to the final segment (`logEffect`) — the `.` is not an identifier
    // char but the assignment target is still a meaningful name.
    let mut tokens: Vec<&str> = before.split_whitespace().collect();
    while let Some(last) = tokens.last() {
        let last = last.trim_end_matches('=').trim();
        if last.is_empty() || last == "=" {
            tokens.pop();
            continue;
        }
        // Normalize `this.logEffect` / `obj.logEffect` → `logEffect`.
        let last = last.rsplit('.').next().unwrap_or(last);
        if last.is_empty() {
            tokens.pop();
            continue;
        }
        if !last.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
            return None;
        }
        return Some(last.to_string());
    }
    None
}

/// Extract signal declarations matching a pattern.
fn extract_signal_decls(
    source: &str,
    shape: &mut SignalShape,
    pattern: &str,
    kind: SignalKind,
) {
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find(pattern) {
        let abs_idx = search_from + idx;

        // Skip matches inside comment lines (`// ...` or `* ...`).
        let line_start = source[..abs_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..abs_idx];
        let line_trim = line.trim_start();
        if line_trim.starts_with("//") || line_trim.starts_with('*') {
            search_from = abs_idx + pattern.len();
            continue;
        }

        let before = &source[..abs_idx];
        let name = extract_decl_name(before).unwrap_or_else(|| "?".to_string());

        // Extract type param from `<T>` in the call
        let after_start = abs_idx + pattern.len();
        let after = &source[after_start..];
        let type_param = if after.starts_with('<') {
            after.find('>').map(|end| {
                let tp = &after[1..end];
                tp.trim().to_string()
            })
        } else {
            None
        };

        shape.signals.push(SignalDecl {
            name,
            kind,
            type_param,
        });

        // Advance past this declaration
        search_from = after_start + 1;
    }
}

/// Extract `effect()` registrations while guarding against the NgRx
/// `createEffect(` collision.
///
/// The bare `effect(` pattern matches inside `createEffect(`, so we must
/// reject occurrences where the character before `effect` is an identifier
/// character (`createEffect`, `useEffect`, `myEffect`), a `.` (method
/// call), or `$`/`_` (prefixed names). A genuine Angular Signals
/// `effect()` call is either:
///   - assigned: `this.logEffect = effect(() => ...)`
///   - standalone in a constructor: `effect(() => ...)`
///   - injected in a function body: `effect(() => ...)`
///
/// In all of these, the character preceding `effect` is whitespace, `=`,
/// `(`, `,`, `;`, or the start of the file — never an identifier char.
fn extract_effect_decls(source: &str, shape: &mut SignalShape) {
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("effect(") {
        let abs_idx = search_from + idx;

        // Skip matches inside comment lines (`// ...` or `* ...`).
        let line_start = source[..abs_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..abs_idx];
        let line_trim = line.trim_start();
        if line_trim.starts_with("//") || line_trim.starts_with('*') {
            search_from = abs_idx + "effect(".len();
            continue;
        }

        // Skip when preceded by an identifier char, `.`, or `$`/`_` —
        // this rejects `createEffect(`, `useEffect(`, `myEffect(`,
        // `obj.effect(`, `_effect(`.
        let prev = source[..abs_idx].chars().last();
        if let Some(c) = prev {
            if c.is_alphanumeric() || matches!(c, '_' | '$' | '.') {
                search_from = abs_idx + "effect(".len();
                continue;
            }
        }

        // Extract the variable name (if this is `name = effect(`).
        let before = &source[..abs_idx];
        let name = extract_decl_name(before).unwrap_or_else(|| "?".to_string());

        // Extract type param from `<T>` in the call (rare for effect).
        let after_start = abs_idx + "effect(".len();
        let after = &source[after_start..];
        let type_param = if after.starts_with('<') {
            after.find('>').map(|end| {
                let tp = &after[1..end];
                tp.trim().to_string()
            })
        } else {
            None
        };

        shape.signals.push(SignalDecl {
            name,
            kind: SignalKind::SignalEffect,
            type_param,
        });

        // Advance past this effect registration.
        search_from = after_start + 1;
    }
}

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Expand every recognised Signals `Φ` marker in a line back to its
/// human-readable form.
pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<SignalKind>(line)
}

/// Expand a single Signals `Φ` marker token.
pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<SignalKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/signals.rs"]
mod tests;