//! Authorized E1 experiment: isolated prototype, not a public query or v2 implementation.
use super::{
    Resolver, Witness, e1_metrics as metrics,
    syntax::{Binding, Unit},
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    collections::BTreeSet,
    path::Path,
    time::{Duration, Instant},
};
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn observe(source: &str, tree: &tree_sitter::Tree) -> Vec<Value> {
    let mut pending = vec![tree.root_node()];
    let mut out = Vec::new();
    while let Some(n) = pending.pop() {
        let mut cursor = n.walk();
        pending.extend(n.named_children(&mut cursor));
        if n.kind() != "object" {
            continue;
        }
        let pairs: Vec<_> = n
            .named_children(&mut cursor)
            .filter(|n| n.kind() == "pair")
            .collect();
        let field = |name: &str| -> Option<tree_sitter::Node<'_>> {
            let matches: Vec<_> = pairs
                .iter()
                .filter(|p| {
                    p.child_by_field_name("key")
                        .is_some_and(|k| k.utf8_text(source.as_bytes()).ok() == Some(name))
                })
                .collect();
            (matches.len() == 1)
                .then(|| matches[0].child_by_field_name("value"))
                .flatten()
        };
        let (Some(path), Some(resolve)) = (field("path"), field("resolve")) else {
            continue;
        };
        if path.kind() != "string" || resolve.kind() != "object" {
            continue;
        }
        let path_text = &source[path.byte_range()];
        let Some(decoded_path) = super::string_value::decode(path_text) else {
            continue;
        };
        let mut c = resolve.walk();
        for pair in resolve
            .named_children(&mut c)
            .filter(|n| n.kind() == "pair")
        {
            let (Some(key), Some(value)) = (
                pair.child_by_field_name("key"),
                pair.child_by_field_name("value"),
            ) else {
                continue;
            };
            if key.kind() != "property_identifier" || value.kind() != "identifier" {
                continue;
            }
            out.push(json!({"predicate":"source_mapping_observation", "class":"proven",
                "route_path":decoded_path, "mapping_key":&source[key.byte_range()],
                "expression":&source[value.byte_range()], "route_span":[n.start_byte(),n.end_byte()],
                "path_span":[path.start_byte(),path.end_byte()], "key_span":[key.start_byte(),key.end_byte()],
                "expression_span":[value.start_byte(),value.end_byte()],
                "qualification":"Written source shape only; no Angular or runtime identity asserted"}));
        }
    }
    out
}

fn write_event(
    out: &Path,
    worker: usize,
    name: &str,
    started: Instant,
    work: impl FnOnce() -> Value,
) -> Value {
    metrics::reset();
    let before = Instant::now();
    let active = ACTIVE.fetch_add(1, Ordering::SeqCst) + 1;
    PEAK.fetch_max(active, Ordering::SeqCst);
    let evidence = work();
    ACTIVE.fetch_sub(1, Ordering::SeqCst);
    let value = json!({"stage":name,"worker":worker,"stage_ms":before.elapsed().as_secs_f64()*1000.0,
        "request_ms":started.elapsed().as_secs_f64()*1000.0,"metrics":metrics::finish(),"evidence":evidence});
    std::fs::write(
        out.join(format!("worker-{worker}-{name}.json")),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    value
}

fn worker(root: &Path, out: &Path, worker: usize, seconds: u64, start: Instant) -> Value {
    let deadline = Duration::from_secs(seconds);
    let expansion = deadline.saturating_sub(Duration::from_millis(100));
    let file = root.join("app.ts");
    let mut observations = Vec::new();
    let mut own_hash = String::new();
    let mut source_unit = None;
    let one = write_event(out, worker, "1-source", start, || {
        let source = std::fs::read_to_string(&file).unwrap();
        metrics::read(&file, source.len(), false);
        own_hash = crate::cache::content_hash(source.as_bytes());
        let unit = Unit::parse(&file, &source).unwrap();
        observations = observe(&source, unit.tree.as_ref().unwrap());
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0]["mapping_key"], "user");
        assert_eq!(observations[0]["expression"], "UserResolver");
        assert_eq!(metrics::snapshot().dependency_parses, 0);
        source_unit = Some(unit);
        json!({"source_hash":own_hash,"observations":observations,"whole_program_attempts":0,
            "automatic_type_traversal":0,"library_traversal":0})
    });
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    let mut endpoint = None;
    let mut target = None;
    let mut entry_witness = None;
    let two = write_event(out, worker, "2-direct-chain", start, || {
        if start.elapsed() >= expansion {
            return json!({"status":"budget_exhausted"});
        }
        target = source_unit
            .as_ref()
            .unwrap()
            .locals
            .get("UserResolver")
            .and_then(|bindings| match bindings.as_slice() {
                [Binding::Declaration(id, _)] => Some(id.clone()),
                _ => None,
            });
        let entry = match resolver.module(&file, "@angular/router") {
            Ok(p) => p,
            Err(e) => return json!({"status":"unresolved","reason":e}),
        };
        if let Some(text) = resolver.host.read(&entry) {
            entry_witness = Some(Witness::Content {
                path: entry.clone(),
                hash: crate::cache::content_hash(text.as_bytes()),
            });
            if let Some(unit) = Unit::parse(&entry, &text) {
                endpoint =
                    unit.exports
                        .get("Routes")
                        .and_then(|bindings| match bindings.as_slice() {
                            [Binding::Declaration(id, _)] => Some(id.clone()),
                            _ => None,
                        });
            }
        }
        json!({"status":if endpoint.is_some() && target.is_some() {"direct_chain_established"} else {"unresolved"},
            "physical_public_entry":entry,"annotation_endpoint":endpoint,"runtime_target":target,
            "qualification":"Relevant ambient competitor closure is not yet established", "whole_program_attempts":0})
    });
    let mut retained_program = None;
    let mut closure = "not_attempted";
    let three = write_event(out, worker, "3-competitor-closure", start, || {
        if start.elapsed() >= expansion {
            return json!({"status":"budget_exhausted", "whole_program_attempts":0});
        }
        std::fs::write(
            out.join(format!("worker-{worker}-acquisition-started.json")),
            serde_json::to_vec(
                &json!({"request_ms":start.elapsed().as_secs_f64()*1000.0,"attempts":1}),
            )
            .unwrap(),
        )
        .unwrap();
        match resolver.program_units(&file) {
            Err(reason) => {
                closure = "unresolved";
                json!({"status":closure,"reason":reason,"whole_program_attempts":1})
            }
            Ok(units) => {
                let ambient =
                    resolver.ambient_scope_in_units(&file, "@angular/router", units.values());
                closure = match &ambient {
                    Ok(None) => "fixture_competitor_scope_checked",
                    Ok(Some(_)) => "conflicting_ambient_scope",
                    Err(_) => "unresolved",
                };
                let count = units.len();
                retained_program = Some(units);
                json!({"status":closure,"program_unit_count":count,"whole_program_attempts":1,
                    "canonical_claim":"not_produced: prototype does not implement full canonical admission"})
            }
        }
    });
    let before_classify = metrics::snapshot();
    let interpretation = if closure == "conflicting_ambient_scope" {
        "partial"
    } else if endpoint.is_some() && target.is_some() {
        "strongly_supported"
    } else {
        "partial"
    };
    assert_eq!(
        before_classify,
        metrics::snapshot(),
        "classification performed I/O"
    );
    let mut publication = Value::Null;
    let four = write_event(out, worker, "4-final-validation", start, || {
        let own = Witness::Content {
            path: file,
            hash: own_hash.clone(),
        };
        let own_current = own.current();
        let mut checked = BTreeSet::new();
        let valid = resolver
            .host
            .witnesses
            .iter()
            .filter(|w| checked.insert(serde_json::to_string(w).unwrap()))
            .all(Witness::current)
            && entry_witness.as_ref().is_none_or(Witness::current);
        let within_budget = start.elapsed() <= deadline;
        publication = json!({"experimental_only":true,"source_observations":if own_current && within_budget {observations.clone()} else {vec![]},
            "interpretation_class":if valid && within_budget {interpretation} else {"unknown"},
            "reason":if !within_budget {"deadline_exhausted; unvalidated-for-budget claims withheld"} else if !valid {"support_invalid"} else {"fixture evidence only"},
            "production_publications":0,"durable_writes":0,"canonical_registrations":[]});
        let mut seen = BTreeSet::new();
        let retained_source_bytes: usize = resolver
            .host
            .sources
            .values()
            .flatten()
            .filter(|s| seen.insert(std::sync::Arc::as_ptr(s) as *const () as usize))
            .map(|s| s.len())
            .sum();
        json!({"own_current":own_current,"direct_support_current":valid,"within_deadline":within_budget,
            "publication":publication,"retained_host_source_bytes":retained_source_bytes,
            "retained_hash_text_bytes":own_hash.len(),
            "serialized_witness_bytes":serde_json::to_vec(&*resolver.host.witnesses).unwrap().len(),
            "retained_program_units":retained_program.as_ref().map_or(0, |p|p.len()),
            "memory_note":"External process working set measured; heap ownership not inferred from this partial payload count"})
    });
    json!({"worker":worker,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"deadline_ms":seconds*1000,
        "stages":[one,two,three,four],"publication":publication})
}

#[test]
fn ops_r1_e1_staged_observation() {
    let Ok(root) = std::env::var("OPS_E1_WORKSPACE") else {
        return;
    };
    let out = std::path::PathBuf::from(std::env::var("OPS_E1_OUTPUT").unwrap());
    let seconds: u64 = std::env::var("OPS_E1_SECONDS").unwrap().parse().unwrap();
    let tasks: usize = std::env::var("OPS_E1_TASKS").unwrap().parse().unwrap();
    assert!([1, 2, 5].contains(&seconds) && [1, 2].contains(&tasks));
    let start = Instant::now();
    ACTIVE.store(0, Ordering::SeqCst);
    PEAK.store(0, Ordering::SeqCst);
    let workers: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..tasks)
            .map(|id| {
                let root = Path::new(&root);
                let out = &out;
                scope.spawn(move || worker(root, out, id, seconds, start))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let result = json!({"category":"E1 test-only staged prototype, not v2 or v1 public query", "workers":workers,
        "elapsed_ms":elapsed,"deadline_ms":seconds*1000,"deadline_overshoot_ms":(elapsed-seconds as f64*1000.0).max(0.0),
        "configured_task_limit":tasks,"actual_worker_count":tasks,"measured_active_stage_peak":PEAK.load(Ordering::SeqCst),
        "request_budget_limitation":"Prototype deadline checks; no hard byte reservations or shared service admission"});
    std::fs::write(
        out.join("completed.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

#[test]
fn ops_r1_e1_observation_scope_controls() {
    let source = "const routes = [{path:'users', resolve:{user:UserResolver}}];";
    let unit = Unit::parse(Path::new("app.ts"), source).unwrap();
    assert_eq!(observe(source, unit.tree.as_ref().unwrap()).len(), 1);
    let duplicate = "const routes = [{path:'a', path:'b', resolve:{user:UserResolver}}];";
    let unit = Unit::parse(Path::new("app.ts"), duplicate).unwrap();
    assert!(observe(duplicate, unit.tree.as_ref().unwrap()).is_empty());
    assert!(Unit::parse(Path::new("bad.ts"), "const routes = [{").is_none());
}
