// RxJS pipe-chain and static-combinator extraction.

use super::{CombinatorDecl, ObservableDecl, PipeChain, PipeOperator, RxJsKind, RxShape};
use crate::meta_util::LexicalRegions;

/// Structural pipe discovery and ordered argument boundaries. Owner resolution
/// remains the existing RxJS textual policy, outside this migrated slice.
pub(super) fn extract_pipe_chains(
    source: &str,
    shape: &mut RxShape,
    lexical_regions: &LexicalRegions,
    evidence: &crate::ir::invocation_evidence::InvocationEvidenceView<'_>,
) {
    use crate::ir::invocation_evidence::WrittenCallee;
    let mut pipes = evidence
        .occurrences()
        .iter()
        .filter_map(|call| {
            let WrittenCallee::Member { receiver, terminal } = call.callee() else {
                return None;
            };
            (evidence.slice(*terminal).ok()? == "pipe").then_some((call, *receiver, *terminal))
        })
        .collect::<Vec<_>>();
    pipes.sort_by_key(|(_, _, terminal)| terminal.start_byte());
    let mut consumed_until = 0;
    for (call, receiver, terminal) in pipes {
        // Preserve legacy outer-body advancement: a pipe inside an operator
        // callback is not separately emitted; receiver chains still are.
        if terminal.start_byte() < consumed_until {
            continue;
        }
        consumed_until = call.span().end_byte();
        let owner = pipe_owner(source, receiver.end_byte(), lexical_regions);
        let operators = call
            .arguments()
            .iter()
            .filter_map(|argument| {
                evidence
                    .slice(argument.span())
                    .ok()
                    .and_then(parse_operator)
            })
            .collect::<Vec<_>>();
        if operators.is_empty() {
            continue;
        }
        if owner != "?" && !shape.observables.iter().any(|o| o.name == owner) {
            shape.observables.push(ObservableDecl {
                name: owner.clone(),
                source: None,
                type_param: None,
            });
        }
        shape.pipes.push(PipeChain { owner, operators });
    }
}

fn pipe_owner(source: &str, pipe_start: usize, lexical_regions: &LexicalRegions) -> String {
    let before = &source[..pipe_start];
    let statement_start = before.rfind([';', '{', '}']).map_or(0, |index| index + 1);
    let statement = before[statement_start..].trim();

    if let Some(eq_index) = statement.rfind('=') {
        if let Some(owner) = assignment_owner(&statement[..eq_index]) {
            return owner;
        }
    }

    enclosing_method_name(source, pipe_start, lexical_regions)
        .or_else(|| expression_owner(statement))
        .unwrap_or_else(|| "?".to_string())
}

fn assignment_owner(lhs: &str) -> Option<String> {
    let name_part = lhs.split(':').next().unwrap_or(lhs).trim();
    let candidate = name_part.split_whitespace().rfind(|word| {
        !matches!(
            *word,
            "private" | "public" | "protected" | "readonly" | "static" | "const" | "let" | "var"
        )
    })?;
    let candidate = candidate.trim();
    (!candidate.is_empty() && candidate != "=" && candidate != ":").then(|| candidate.to_string())
}

fn expression_owner(statement: &str) -> Option<String> {
    statement
        .split_whitespace()
        .next_back()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "return" && *value != ")" && *value != "}")
        .map(str::to_string)
}

fn enclosing_method_name(
    source: &str,
    position: usize,
    lexical_regions: &LexicalRegions,
) -> Option<String> {
    let bytes = source.as_bytes();
    let mut cursor = position;
    let mut closed_depth = 0usize;

    while cursor > 0 {
        cursor -= 1;
        match bytes[cursor] {
            b'}' if !lexical_regions.contains(cursor) => {
                closed_depth += 1;
            }
            b'{' if !lexical_regions.contains(cursor) => {
                if closed_depth > 0 {
                    closed_depth -= 1;
                } else if let Some(name) = method_name_before_brace(source, cursor) {
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    None
}

fn method_name_before_brace(source: &str, brace: usize) -> Option<String> {
    let prefix = &source[..brace];
    let head_start = prefix.rfind([';', '{', '}']).map_or(0, |index| index + 1);
    let head = prefix[head_start..].trim();
    let (params_open, _) = crate::compaction::method::find_method_params(head)?;
    let parts = crate::compaction::signature::split_head_parts(head, params_open)?;
    if matches!(
        parts.bare_name,
        "if" | "for" | "while" | "switch" | "catch" | "with"
    ) {
        return None;
    }
    Some(parts.bare_name.to_string())
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
            let mut end = 37;
            while !args.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &args[..end])
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
pub(super) fn extract_combinators(
    source: &str,
    shape: &mut RxShape,
    lexical_regions: &LexicalRegions,
) {
    let combinator_names = ["combineLatest", "forkJoin", "merge", "zip", "race"];

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

            // Round-11 audit: reject matches inside trailing comments, block
            // comments, or string literals (e.g. a `combineLatest(` inside a
            // string literal or a trailing `// combineLatest(...)` comment).
            if lexical_regions.contains(abs_idx) {
                search_from = abs_idx + pattern.len();
                continue;
            }

            // Collect the full call body (up to matching close paren).
            let after_start = abs_idx + pattern.len();
            let (body, end_offset) =
                crate::angular_meta::util::collect_call_body(&source[after_start..]);

            // Extract arguments from the body. Use depth-aware splitting so
            // commas inside object literals or nested calls (e.g.
            // `combineLatest([a$, b$], { ... })`) do NOT fragment the args.
            let args: Vec<String> = crate::angular_meta::util::split_top_level(&body, ',')
                .into_iter()
                .map(|s| s.trim().trim_end_matches(')').trim().to_string())
                .filter(|s| !s.is_empty() && *s != "[" && *s != "]")
                .collect();

            shape.combinators.push(CombinatorDecl {
                kind: RxJsKind::Combine,
                name: name.to_string(),
                args,
            });

            // Advance past the whole call (including its close paren) using
            // the standardized end_offset contract.
            search_from = after_start + end_offset;
        }
    }
}
