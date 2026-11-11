use super::model::{Candidate, Diagnostic, KINDS, RELATIONS, STATUSES};
use std::collections::BTreeMap;

fn node_id(id: &str) -> bool {
    let upper = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    };
    if let Some((prefix, value)) = id.split_once('.') {
        return ["AUTH", "INV", "DEBT"].contains(&prefix)
            && value.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
            && value
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_');
    }
    let parts: Vec<_> = id.split('-').collect();
    if parts[0].starts_with('P') {
        let tail = &parts[0][1..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        if (3..=4).contains(&digits)
            && tail
                .as_bytes()
                .get(digits)
                .is_some_and(u8::is_ascii_uppercase)
            && upper(&tail[digits..])
            && parts[1..].iter().all(|p| upper(p))
        {
            return true;
        }
    }
    parts.len() >= 2
        && (3..=8).contains(&parts[0].len())
        && parts[0].bytes().all(|c| c.is_ascii_uppercase())
        && (parts[1].len() == 3 || parts[1].len() == 4)
        && parts[1].as_bytes()[..3].iter().all(u8::is_ascii_digit)
        && (parts[1].len() == 3 || parts[1].as_bytes()[3].is_ascii_uppercase())
        && parts[2..].iter().all(|p| upper(p))
}

/// Exact GLS candidate-state rules. No pairwise transition or authorization matrix.
pub fn validate(nodes: &BTreeMap<String, Candidate>) -> Vec<Diagnostic> {
    let mut errors = Vec::new();
    let mut superseders = BTreeMap::<&str, usize>::new();
    for edge in nodes
        .values()
        .flat_map(|n| &n.declaration.relationships)
        .filter(|e| e.kind == "SUPERSEDES")
    {
        *superseders.entry(&edge.target).or_default() += 1;
    }
    for (id, node) in nodes {
        let d = &node.declaration;
        let mut problem =
            |code, message| errors.push(Diagnostic::new(code, &node.path, Some(id), message));
        if !node_id(id) {
            problem(
                "E_INVALID_NODE_ID",
                "Use the GLS mission/infrastructure/AUTH/INV/DEBT identity grammar",
            );
        }
        if !KINDS.contains(&d.kind.as_str()) {
            problem("E_INVALID_KIND", "Select one of the seven GLS kinds");
        }
        if !STATUSES.contains(&d.status.as_str()) {
            problem("E_INVALID_STATUS", "Select one of the twelve GLS statuses");
        }
        if [
            "FROZEN",
            "IMPLEMENTED",
            "VALIDATED",
            "RECLOSED",
            "BANKED",
            "FALSIFIED",
            "SUPERSEDED",
        ]
        .contains(&d.status.as_str())
            && d.artifact.is_none()
        {
            problem(
                "E_MISSING_ARTIFACT",
                "Declare the repository evidence artifact required by this status",
            );
        }
        let mut blockers = 0;
        let mut unresolved = 0;
        for edge in &d.relationships {
            if !RELATIONS.contains(&edge.kind.as_str()) {
                problem("E_INVALID_EDGE_TYPE", "Use the GLS directional vocabulary");
            }
            let Some(target) = nodes.get(&edge.target) else {
                problem(
                    "E_UNKNOWN_NODE",
                    "Declare the referenced endpoint before admitting the candidate graph",
                );
                continue;
            };
            if edge.target == *id {
                problem(
                    "E_SELF_EDGE",
                    "A relationship cannot target its declaring node",
                );
                continue;
            }
            let t = &target.declaration;
            let satisfied = ["RECLOSED", "BANKED"].contains(&t.status.as_str());
            match edge.kind.as_str() {
                "DEPENDS_ON" if ["FALSIFIED", "SUPERSEDED"].contains(&t.status.as_str()) => {
                    problem(
                        "E_DEPENDS_ON_INVALID_PROPOSITION",
                        "Historical failed/superseded evidence cannot be a necessary premise",
                    )
                }
                "IMPLEMENTS"
                    if !["MISSION", "IMPLEMENTATION", "AUTHORITY"].contains(&t.kind.as_str()) =>
                {
                    problem(
                        "E_INVALID_EDGE_TARGET_KIND",
                        "IMPLEMENTS targets a contract-bearing MISSION, IMPLEMENTATION or AUTHORITY",
                    )
                }
                "RECLOSES" => {
                    if !["MISSION", "IMPLEMENTATION"].contains(&t.kind.as_str()) {
                        problem(
                            "E_INVALID_EDGE_TARGET_KIND",
                            "RECLOSES targets MISSION or IMPLEMENTATION",
                        );
                    } else if d.status == "RECLOSED" && !satisfied {
                        problem(
                            "E_RECLOSURE_TARGET_NOT_CLOSED",
                            "RECLOSED reclosure requires a RECLOSED/BANKED target",
                        );
                    }
                }
                "SUPERSEDES" if t.status != "SUPERSEDED" => problem(
                    "E_SUPERSESSION_STATUS",
                    "SUPERSEDES requires the target status SUPERSEDED",
                ),
                "PRODUCES_AUTHORITY" | "CONSUMES_AUTHORITY" if t.kind != "AUTHORITY" => problem(
                    "E_INVALID_EDGE_TARGET_KIND",
                    "Authority relations target AUTHORITY",
                ),
                "BLOCKED_BY" => {
                    blockers += 1;
                    if !satisfied {
                        unresolved += 1;
                    }
                }
                _ => {}
            }
        }
        if d.status == "BLOCKED" {
            if blockers == 0 {
                problem(
                    "E_BLOCKED_WITHOUT_BLOCKER",
                    "BLOCKED requires at least one declared blocker",
                );
            } else if unresolved == 0 {
                problem(
                    "E_STALE_BLOCK",
                    "All blockers are satisfied; author must correct the stale BLOCKED declaration",
                );
            }
        }
        if ["IMPLEMENTATION_READY", "VALIDATED", "RECLOSED", "BANKED"].contains(&d.status.as_str())
            && unresolved > 0
        {
            problem(
                "E_UNSATISFIED_PREREQUISITE",
                "Every declared blocker must be RECLOSED or BANKED",
            );
        }
        if d.status == "SUPERSEDED" && superseders.get(id.as_str()).copied().unwrap_or(0) != 1 {
            problem(
                "E_SUPERSEDED_WITHOUT_SUCCESSOR",
                "SUPERSEDED requires exactly one incoming SUPERSEDES",
            );
        }
    }
    errors
}
