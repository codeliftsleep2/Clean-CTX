// src/ir/calls.rs
//
// Generic native call-fact production (`CoreOp::Call`).
//
// This module owns the language-AGNOSTIC half of native call extraction:
//
//   grammar-specific invocation captures  →  THIS module  →  CoreOp::Call
//
// A language producer supplies only two things:
//   1. an invocation-capture query (`call_capture_query(base_query)`), and
//   2. the existing IR pass, which feeds the captures through
//      `CallProducer` while it walks the SAME tree-sitter parse.
//
// Everything downstream — the normalized `CoreOp::Call` fact, explicit
// argument-count semantics, callable-span ownership, and the semantic
// projection — is shared, so a language producer adds a query plus a
// `call_capture_query` mapping entry and nothing else. C#, TypeScript, and Java
// are wired that way today; Rust has no arm and compiles exactly as before.
//
// Truthfulness of the fact (established by the approved investigation):
//   * caller  = the `DefMethod` id of the innermost callable whose source span
//               contains the invocation, in the same file;
//   * callee  = the method NAME as written at the call site (never a resolved
//               declaration identity);
//   * argc    = the number of explicitly written arguments, counted from the
//               individual `argument` nodes captured by the query — never from
//               text splitting, regex, or a second parse.
//
// An invocation that is not inside a precisely identified callable produces NO
// fact. Ownership is never guessed from a class, a file, or a stale
// `current_method`, so an unsupported context (a field initializer, a property
// accessor, an unmatched function shape) yields no edge at all.

use std::collections::{BTreeMap, HashMap};

use super::opcodes::CoreOp;

/// Capture name of the callee name node written at the call site.
pub const CALL_CALLEE_CAPTURE: &str = "call.callee";
/// Capture name of one explicitly written argument node.
pub const CALL_ARGUMENT_CAPTURE: &str = "call.argument";
/// Capture name of one explicitly written argument node that EXPANDS at run
/// time (a TypeScript spread element).
///
/// It is a distinct capture name rather than a flag on `call.argument` because
/// the two carry different facts: an `argument` contributes one written
/// argument to the count, while a `spread` contributes one written argument to
/// the count AND marks the whole invocation's count as non-exact. The producer
/// therefore records them through separate methods and the capture walk stays
/// the single source of both the count and the qualifier.
pub const CALL_SPREAD_CAPTURE: &str = "call.spread";

/// Grammar boundary: the invocation-capture query that the IR pass appends to
/// `base_query` so this producer runs.
///
/// This is the ONLY language-specific thing a producer supplies. C#,
/// TypeScript, and Java each map their base query to their own invocation
/// capture query; the normalized `CoreOp::Call` fact, argument-count semantics,
/// callable-span ownership, the semantic projection, the graph edge, and every
/// workspace consumer are shared and untouched.
///
/// Returns `None` for every language without a native call producer (Rust
/// today), so those languages compile exactly as before.
pub fn call_capture_query(base_query: &str) -> Option<&'static str> {
    if base_query == crate::queries::CS_QUERY {
        Some(crate::queries::CS_CALL_QUERY)
    } else if base_query == crate::queries::TS_QUERY {
        Some(crate::queries::TS_CALL_QUERY)
    } else if base_query == crate::queries::JAVA_QUERY {
        Some(crate::queries::JAVA_CALL_QUERY)
    } else {
        None
    }
}

/// The query string to run for one compilation: `base_query` plus the
/// language's invocation-capture query when a producer exists.
///
/// Both halves are compiled into ONE `Query` and walked in ONE parse, so the
/// call producer never parses the file a second time.
pub fn capture_query(base_query: &str) -> String {
    match call_capture_query(base_query) {
        Some(call_query) => format!("{base_query}\n{call_query}"),
        None => base_query.to_string(),
    }
}

/// Open callable scope: the callable's `DefMethod` id plus the source span of
/// the declaration that owns it.
///
/// Mirrors `TypeScope`: scopes are pushed when the callable declaration capture
/// is processed and pruned once the walk passes `end_byte`, so the active
/// callable is always the innermost declaration whose `[start_byte, end_byte)`
/// window contains the current capture.
#[derive(Debug, Clone)]
pub struct CallableScope {
    /// `DefMethod` id allocated for this callable declaration.
    pub method_id: String,
    /// Byte offset of the start of the callable declaration node.
    pub start_byte: usize,
    /// Byte offset one past the end of the callable declaration node.
    pub end_byte: usize,
}

/// One invocation awaiting its final argument count.
#[derive(Debug, Clone)]
struct PendingCall {
    /// Callee name as written at the call site.
    callee: String,
    /// Explicit argument count observed so far.
    explicit_arg_count: usize,
    /// Whether any observed argument expands at run time, making
    /// `explicit_arg_count` a written-node count rather than an exact arity.
    has_spread: bool,
    /// Innermost callable that owns the invocation, if any.
    owner: Option<String>,
}

/// Producer state for a single compilation.
///
/// Call facts are accumulated while the capture walk is in progress (the
/// argument captures of an invocation can arrive after its callee capture) and
/// emitted when the owning callable's region closes, so the emitted
/// `CoreOp::Call` ops stay adjacent to their caller's instruction region.
#[derive(Debug, Default)]
pub struct CallProducer {
    /// Query match → the invocation it bound, identified by the callee node's
    /// `(start_byte, end_byte)`. Several matches describe one invocation (the
    /// argument-less pattern enumerates it, the argument pattern enumerates its
    /// arity); they all bind the SAME callee node, so they share this key.
    match_callee: HashMap<usize, (usize, usize)>,
    /// Invocation (callee node span) → pending fact, in document order.
    calls: BTreeMap<(usize, usize), PendingCall>,
    /// Invocations dropped because no precise callable owned them.
    unattributed: usize,
    /// Invocations emitted as facts (diagnostic).
    emitted: usize,
}

impl CallProducer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the callee capture of one query match.
    ///
    /// `owner` is the innermost callable containing the invocation at the
    /// moment the capture was seen; the same invocation is recorded only once,
    /// no matter how many matches bind its callee.
    pub fn record_callee(
        &mut self,
        match_index: usize,
        start_byte: usize,
        end_byte: usize,
        callee: &str,
        owner: Option<&str>,
    ) {
        let key = (start_byte, end_byte);
        self.match_callee.insert(match_index, key);
        self.calls.entry(key).or_insert_with(|| PendingCall {
            callee: callee.to_string(),
            explicit_arg_count: 0,
            has_spread: false,
            owner: owner.map(str::to_string),
        });
    }

    /// Record one explicitly written argument of one query match.
    ///
    /// A match that binds arguments also binds its callee, and the pipeline
    /// emits a match's captures callee-first (a field capture precedes the
    /// child captures nested inside its argument list), so the argument always
    /// finds its invocation already recorded. A match with zero arguments
    /// simply contributes nothing, leaving the count at 0.
    pub fn record_argument(&mut self, match_index: usize) {
        if let Some(key) = self.match_callee.get(&match_index)
            && let Some(call) = self.calls.get_mut(key)
        {
            call.explicit_arg_count += 1;
        }
    }

    /// Record one explicitly written argument of one query match that EXPANDS
    /// at run time (a TypeScript spread element).
    ///
    /// It contributes exactly one written argument to the count — a spread
    /// argument is one written node, never a guessed expanded number — and
    /// marks the invocation's count as NOT exact, so no consumer can read the
    /// written count as an arity.
    pub fn record_spread(&mut self, match_index: usize) {
        if let Some(key) = self.match_callee.get(&match_index)
            && let Some(call) = self.calls.get_mut(key)
        {
            call.explicit_arg_count += 1;
            call.has_spread = true;
        }
    }

    /// Emit and drop every pending fact owned by `owner` (in document order),
    /// and drop the pending facts that carry no callable owner.
    ///
    /// Unattributed invocations are never guessed into an edge; they are
    /// counted for diagnostics and disappear.
    pub fn settle(&mut self, owner: Option<&str>) -> Vec<CoreOp> {
        let mut emitted = Vec::new();
        let mut keep: BTreeMap<(usize, usize), PendingCall> = BTreeMap::new();
        for (key, call) in std::mem::take(&mut self.calls) {
            match (call.owner.as_deref(), owner) {
                (Some(call_owner), Some(owner)) if call_owner == owner => {
                    self.emitted += 1;
                    emitted.push(CoreOp::Call(
                        call_owner.to_string(),
                        call.callee,
                        call.explicit_arg_count,
                        call.has_spread,
                    ));
                }
                (None, _) => {
                    self.unattributed += 1;
                }
                _ => {
                    keep.insert(key, call);
                }
            }
        }
        self.calls = keep;
        emitted
    }

    /// Number of invocations dropped for want of a precise callable owner
    /// (diagnostic; see the "do not guess ownership" contract).
    pub fn unattributed(&self) -> usize {
        self.unattributed
    }

    /// Number of invocations still awaiting a settle point.
    pub fn pending(&self) -> usize {
        self.calls.len()
    }

    /// Number of call facts emitted for this compilation (diagnostic).
    pub fn emitted(&self) -> usize {
        self.emitted
    }
}

#[cfg(test)]
#[path = "../tests/ir/calls.rs"]
mod tests;

// The C# production capture path for native call facts (RED-CALL1..RED-CALL13,
// RED-CALL24..RED-CALL26). Declared here because this module owns the producer
// boundary those tests exercise.
#[cfg(all(test, feature = "csharp"))]
#[path = "../tests/ir/calls_csharp.rs"]
mod csharp_tests;

// The TypeScript production capture path for native call facts: the same
// producer boundary, reached through the TypeScript invocation grammar.
#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/ir/calls_typescript.rs"]
mod typescript_tests;

// The Java production capture path for native call facts: the same producer
// boundary, reached through the Java invocation grammar.
#[cfg(all(test, feature = "java"))]
#[path = "../tests/ir/calls_java.rs"]
mod java_tests;
