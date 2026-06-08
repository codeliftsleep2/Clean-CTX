// src/angular_meta/detect.rs
//
// Angular detection heuristic.
//
// The Meta-Layer must run **only** on Angular files. Non-Angular `.ts`
// files (Deno, Node, React, Vue, Svelte, etc.) should pay **zero**
// overhead — no Φ markers, no extra parse, no newlines.
//
// # Strategy
//
// We do not re-parse the AST. We scan the raw source for the
// decorator names that are unique to Angular:
//
//   @Component, @Injectable, @NgModule, @Directive, @Pipe, @Input,
//   @Output, @HostListener, @HostBinding, @ViewChild, @ContentChild,
//   ...
//
// A single occurrence of `@Component(` or `@Injectable(` is a strong
// enough signal. (Plain `@Input` / `@Output` are also used by MobX /
// Vue, so we treat them as weak signals — at least one strong signal
// must also be present.)
//
// The detection is O(n) over the source length and never allocates
// more than a single `BTreeSet` of the matches it found.

/// Angular-specific decorator names that we treat as a strong signal
/// of an Angular file. A single match anywhere in the source is enough
/// to consider the file Angular.
const STRONG_DECORATORS: &[&str] = &[
    "@Component(",
    "@Directive(",
    "@Injectable(",
    "@NgModule(",
    "@Pipe(",
    "@HostListener(",
    "@HostBinding(",
    "@ViewChild(",
    "@ViewChildren(",
    "@ContentChild(",
    "@ContentChildren(",
];

/// Angular-specific decorator names that are NOT unique to Angular
/// (e.g. `@Input` is also used by MobX, `@Output` by Vue). These count
/// as weak signals and must be paired with a strong signal to trigger
/// Meta-Layer output.
const WEAK_DECORATORS: &[&str] = &["@Input(", "@Input ", "@Output(", "@Output "];

/// Detects the presence of the Angular `core` import. Almost every
/// Angular file imports something from `@angular/core`.
const ANGULAR_CORE_IMPORT: &str = "@angular/core";

/// Decide whether the given source code is from an Angular file.
///
/// A file is "Angular" iff:
/// 1. It contains at least one **strong** decorator (`@Component(`,
///    `@Injectable(`, `@NgModule(`, `@Directive(`, `@Pipe(`, etc.), OR
/// 2. It imports from `@angular/core` AND has at least one
///    `@Input` / `@Output` decorator (weak pair).
///
/// Plain `@Input` / `@Output` alone (no strong signal, no
/// `@angular/core` import) returns `false` — those decorators are
/// also used by MobX / Vue, and a false positive would inject
/// meaningless `Φ` markers into non-Angular output.
pub fn is_angular_file(source: &str) -> bool {
    // Tier 1: any strong decorator?
    for deco in STRONG_DECORATORS {
        if source.contains(deco) {
            return true;
        }
    }

    // Tier 2: `@angular/core` import + weak decorator pair?
    if source.contains(ANGULAR_CORE_IMPORT) {
        for weak in WEAK_DECORATORS {
            if source.contains(weak) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
#[path = "../tests/angular_meta/detect.rs"]
mod tests;
