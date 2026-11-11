//! Closed v1 payload shaping and optional-first packing with shared citation fragments.
use super::*;
use crate::pcg::{
    security,
    wire::{self, Fragment},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn records(values: &[Value]) -> Vec<Value> {
    values
        .iter()
        .map(|v| {
            json!({"id":v["id"],"revision":v["revision"],"role":v["role"],
        "required":v["required"],"authority":v["authority"],"verification":v["verification"],
        "lifecycle":v["lifecycle"],"freshness":v["freshness"],"subject":v["subject"],
        "disposition":v["disposition"],"citations":v["citations"]})
        })
        .collect()
}
fn edge(v: &Value) -> Value {
    json!({"id":v["id"],"revision":v["revision"],"kind":v["kind"],
        "from":v["from"],"to":v["to"],"citation":v["citation"]})
}
fn issue(code: &str) -> Value {
    let code = match code {
        "history_required" => "source_inaccessible",
        other => other,
    };
    json!({"code":code,"records":[]})
}
struct Status<'a> {
    state: &'a str,
    fit: bool,
    truncated: bool,
    completion: bool,
    overflow: bool,
}
fn fields(
    a: &Acquisition,
    q: &Request,
    trusted: &Trusted,
    meter: &Meter<'_>,
    status: Status<'_>,
    omissions: &BTreeSet<String>,
) -> Result<BTreeMap<&'static str, Fragment>> {
    let Status {
        state,
        fit,
        truncated,
        completion,
        overflow,
    } = status;
    let c = &a.coverage;
    let reasons: Vec<_> = if overflow {
        vec![issue("required_budget_exceeded")]
    } else {
        a.reasons.iter().map(|s| issue(s)).collect()
    };
    let value = json!({"version":1,"scope":{"partition":trusted.profile.partition_id,
        "corpus":q.corpus,"workspace":q.subject.workspace},
        "generation":if a.state == State::Unavailable {Value::Null} else {json!(a.generation)},
        "authority_policy":trusted.profile.policy_revision,
        "subjects":[identity::canonical(&serde_json::to_value(&q.subject).map_err(|_| Error("wire_encoding"))?)],
        "required_records":[],"support":[],"history":[],"relationships":[],"citations":[],
        "conflicts":[],"missing_stale_requirements":[],"reasons":reasons,"result_state":state,
        "coverage":{"enrollment":c.enrollment,"identity":c.identity,"applicability":c.applicability,
            "required_closure":c.required_closure,"evidence":c.evidence,
            "currentness_boundary":"pinned_revision_two_observations","freshness_guarantee":"observed_boundaries_only",
            "continuous_currentness":false,"atomic_filesystem_snapshot":false,
            "projection_integrity":"admission_and_accessed_rows","observations":{
                "working_start":observation(c.working_start, a, false),
                "working_completion":if completion {"matched"} else {observation(c.working_completion, a, true)}}},
        "budget":{"limit":meter.limit,"tokenizer":meter.name,"accounting":"serialized_mcp_result_v1",
            "required_material_fit":fit,"optional_truncated":truncated,"diagnostics_truncated":false,
            "optional_omissions":omissions}});
    security::json_nonsecret(&value)?;
    value
        .as_object()
        .ok_or(Error("wire_encoding"))?
        .iter()
        .map(|(k, v)| Ok((field_name(k)?, Fragment::value(v)?)))
        .collect()
}

// Keys are closed, static v1 names; no untrusted key can extend the wire contract.
fn field_name(key: &str) -> Result<&'static str> {
    const KEYS: &[&str] = &[
        "version",
        "scope",
        "generation",
        "authority_policy",
        "subjects",
        "required_records",
        "support",
        "history",
        "relationships",
        "citations",
        "conflicts",
        "missing_stale_requirements",
        "reasons",
        "result_state",
        "coverage",
        "budget",
    ];
    KEYS.iter()
        .copied()
        .find(|k| *k == key)
        .ok_or(Error("wire_encoding"))
}
fn observation(matched: bool, a: &Acquisition, completion: bool) -> &'static str {
    if matched {
        "matched"
    } else if a.reasons.contains("work_limit") {
        "incomplete"
    } else if a.reasons.iter().any(|s| s.starts_with("source_"))
        && (!completion || a.coverage.working_start)
    {
        "failed"
    } else {
        "not_checked"
    }
}
fn encode(values: &[Value]) -> Result<Vec<Fragment>> {
    values
        .iter()
        .map(|v| {
            security::json_nonsecret(v)?;
            Fragment::value(v)
        })
        .collect()
}

pub(super) fn prepare(
    a: &Acquisition,
    q: &Request,
    trusted: &Trusted,
    mut support: optional::Support,
    meter: &mut Meter<'_>,
) -> Result<(String, usize)> {
    let required = records(&a.required_records);
    let mut optional = records(&support.records);
    let mut history = records(&support.history);
    optional.sort_by_key(|r| r["id"].as_str().unwrap_or("").to_owned());
    history.sort_by_key(|r| r["id"].as_str().unwrap_or("").to_owned());
    let required_fragments = encode(&required)?;
    let optional_fragments = encode(&optional)?;
    let history_fragments = encode(&history)?;
    let mut all_edges: Vec<_> = a
        .relationships
        .iter()
        .chain(support.relationships.iter())
        .map(edge)
        .collect();
    all_edges.sort_by_key(|v| {
        (
            v["kind"].to_string(),
            v["from"].to_string(),
            v["to"].to_string(),
            v["id"].to_string(),
        )
    });
    let edge_fragments = encode(&all_edges)?;
    let mut citations = BTreeMap::new();
    let mut execution_index: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for r in a
        .required_records
        .iter()
        .chain(support.records.iter())
        .chain(support.history.iter())
    {
        if let (Some(primary), Some(execution)) = (
            r["primary"].as_str(),
            r["verification"]["execution"].as_str(),
        ) {
            execution_index
                .entry(primary)
                .or_default()
                .insert(execution);
        }
    }
    for v in a.citations.iter().chain(support.citations.iter()) {
        let executions = execution_index
            .get(v["id"].as_str().unwrap_or(""))
            .cloned()
            .unwrap_or_default();
        let execution = if executions.len() == 1 {
            executions.iter().next().copied().unwrap_or("not_run")
        } else {
            "not_run"
        };
        let citation = json!({"id":v["id"],"source":v["source"],"commit":v["commit"],"blob":v["blob"],
            "source_sha256":v["source_sha256"],"start":v["start"],"end":v["end"],
            "quote_sha256":v["quote_sha256"],"text":v["text"],"freshness":v["freshness"],"execution":execution});
        security::json_nonsecret(&citation)?;
        let id = v["id"]
            .as_str()
            .ok_or(Error("projection_corrupt"))?
            .to_owned();
        let order = (
            v["source_id"].to_string(),
            v["source_revision"].to_string(),
            v["start"].as_u64(),
            v["end"].as_u64(),
        );
        require(!citations.contains_key(&id), "projection_corrupt")?;
        citations.insert(id, (order, Fragment::value(&citation)?));
        meter.counts.citation_encodings += 1;
    }
    let required_citations: BTreeSet<_> = a
        .citations
        .iter()
        .filter_map(|v| v["id"].as_str())
        .collect();
    let mut n = optional.len();
    let mut h = history.len();
    let mut standalone: Vec<_> = support.standalone.into_iter().collect();
    let mut truncated = support.omissions.contains("work_limit");
    loop {
        let selected: BTreeSet<_> = required
            .iter()
            .chain(optional[..n].iter())
            .chain(history[..h].iter())
            .filter_map(|r| r["id"].as_str())
            .collect();
        let selected_edges: Vec<_> = all_edges
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                selected.contains(e["from"].as_str().unwrap_or(""))
                    && (e["kind"] == "CONSTRAINS"
                        || selected.contains(e["to"].as_str().unwrap_or("")))
            })
            .collect();
        let mut wanted = required_citations.clone();
        wanted.extend(standalone.iter().map(String::as_str));
        for r in optional[..n].iter().chain(history[..h].iter()) {
            wanted.extend(
                r["citations"]
                    .as_array()
                    .ok_or(Error("projection_corrupt"))?
                    .iter()
                    .filter_map(Value::as_str),
            );
        }
        wanted.extend(
            selected_edges
                .iter()
                .filter_map(|(_, e)| e["citation"].as_str()),
        );
        let mut selected_citations: Vec<_> = citations
            .iter()
            .filter(|(id, _)| wanted.contains(id.as_str()))
            .map(|(_, v)| v)
            .collect();
        require(
            selected_citations.len() == wanted.len(),
            "projection_corrupt",
        )?;
        selected_citations.sort_by(|a, b| a.0.cmp(&b.0));
        let mut fields = fields(
            a,
            q,
            trusted,
            meter,
            Status {
                state: "complete_within_registered_scope",
                fit: true,
                truncated,
                completion: true,
                overflow: false,
            },
            &support.omissions,
        )?;
        fields.insert(
            "required_records",
            Fragment::array(required_fragments.iter()),
        );
        fields.insert("support", Fragment::array(optional_fragments[..n].iter()));
        fields.insert("history", Fragment::array(history_fragments[..h].iter()));
        fields.insert(
            "relationships",
            Fragment::array(selected_edges.iter().map(|(i, _)| &edge_fragments[*i])),
        );
        fields.insert(
            "citations",
            Fragment::array(selected_citations.into_iter().map(|v| &v.1)),
        );
        let serialized = wire::result(fields)?;
        let used = meter.count(&serialized);
        if meter.fits(used) {
            return Ok((serialized, used));
        }
        if h > 0 {
            h -= 1;
        } else if n > 0 {
            n -= 1;
        } else if !standalone.is_empty() {
            standalone.pop();
        } else {
            return control(a, q, trusted, meter, true);
        }
        truncated = true;
        support.omissions.insert("budget".into());
    }
}

pub(super) fn control(
    a: &Acquisition,
    q: &Request,
    trusted: &Trusted,
    meter: &mut Meter<'_>,
    overflow: bool,
) -> Result<(String, usize)> {
    let state = if overflow {
        "blocked"
    } else {
        match a.state {
            State::Unavailable => "unavailable",
            State::Blocked => "blocked",
            _ => "incomplete",
        }
    };
    let omissions = BTreeSet::from(["not_requested".into()]);
    let serialized = wire::result(fields(
        a,
        q,
        trusted,
        meter,
        Status {
            state,
            fit: false,
            truncated: false,
            completion: overflow || a.coverage.working_completion,
            overflow,
        },
        &omissions,
    )?)?;
    let used = meter.count(&serialized);
    require(meter.fits(used), "token_budget_too_small")?;
    Ok((serialized, used))
}
