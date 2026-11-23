//! Read produced, admitted registrations; never interpret TS mappings at query time.
#[cfg(feature = "angular")]
use super::outcome::QueryAnswer;
use super::outcome::QueryFailure;
use super::prepare::{PreparationContext, PreparedQuery};
use crate::mcp::McpState;
use serde_json::Value;
#[cfg(feature = "angular")]
use serde_json::json;

pub(super) fn prepare(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> PreparedQuery {
    match prepare_inner(args, state, context) {
        Ok(prepared) => prepared,
        Err(error) => PreparedQuery::failure(error),
    }
}

fn prepare_inner(
    args: &Value,
    state: &McpState,
    context: &mut PreparationContext,
) -> Result<PreparedQuery, QueryFailure> {
    let root = args["workspaceRoot"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| QueryFailure::invalid("resolver_registrations requires workspaceRoot"))?;
    let direction = args["direction"]
        .as_str()
        .ok_or_else(|| QueryFailure::invalid("direction must be forward or reverse"))?;
    let (field, opposite) = match direction {
        "forward" => ("route_path", "resolver_name"),
        "reverse" => ("resolver_name", "route_path"),
        _ => {
            return Err(QueryFailure::invalid(
                "direction must be forward or reverse",
            ));
        }
    };
    if args.get(opposite).is_some() {
        return Err(QueryFailure::invalid(
            "Opposite resolver selector is not permitted",
        ));
    }
    let selector = args[field]
        .as_str()
        .filter(|s| direction == "forward" || !s.is_empty())
        .ok_or_else(|| QueryFailure::invalid(format!("Invalid required {field}")))?
        .to_owned();
    let scope = context.scope(state, args)?;
    if !crate::layers::meta::resolver_registrations::enabled(&state.config) {
        return Err(QueryFailure::invalid(
            "Written resolver registration production is disabled",
        ));
    }
    #[cfg(feature = "angular")]
    {
        let nomination = context
            .resolver_nomination
            .get_or_insert_with(|| super::super::hydration::hydrate_resolver_owners(state, root))
            .clone()
            .map_err(QueryFailure::internal)?;
        let footprints = {
            let index = state.workspace_index_read();
            nomination
                .owners
                .iter()
                .map(|owner| {
                    if !index.has_resolver_registration_admission(owner) {
                        return Err(QueryFailure::internal(
                            "Required registration projection is missing; reacquire source",
                        ));
                    }
                    let hash = index
                        .semantic_projection_source_hash(owner)
                        .ok_or_else(|| {
                            QueryFailure::internal("Required semantic owner is missing")
                        })?;
                    Ok(crate::layers::meta::semantic::SemanticDependency {
                        file_path: owner.clone(),
                        source_hash: Some(hash.to_owned()),
                    })
                })
                .collect::<Result<Vec<_>, QueryFailure>>()?
        };
        let direction = direction.to_owned();
        Ok(PreparedQuery::indexed(move |index| {
            // Source ownership stays independent; exact filesystem constraints
            // share work only within this final-consumption evaluation.
            let sources_current = footprints.iter().all(|proof| {
                #[cfg(test)]
                super::work_metrics::record_resolver_owner_validation();
                proof.is_current() && index.has_resolver_registration_admission(&proof.file_path)
            });
            let projections: Vec<_> = footprints
                .iter()
                .map(|proof| {
                    (
                        proof.file_path.as_str(),
                        crate::workspace::index::SemanticFidelity::High,
                        proof.source_hash.as_deref().unwrap_or(""),
                    )
                })
                .collect();
            if !sources_current || !index.has_current_semantic_projections(&projections) {
                return Err(QueryFailure::internal(
                    "Registration source/dependency publication changed; retry",
                ));
            }
            let mut records = Vec::new();
            let mut selected = Vec::new();
            let mut unassociated = Vec::new();
            for owner in &nomination.owners {
                if scope.as_ref().is_some_and(|scope| !scope.admits(owner)) {
                    continue;
                }
                let Some(collection) = index.resolver_registrations(owner) else {
                    continue;
                };
                #[cfg(test)]
                super::work_metrics::record_resolver_records(collection.records.len());
                records.extend(
                    collection
                        .records
                        .iter()
                        .filter(|record| {
                            if direction == "forward" {
                                record.route_path == selector
                            } else {
                                record.resolver.name == selector
                            }
                        })
                        .cloned(),
                );
                for omission in &collection.semantic_omissions {
                    let associated = if direction == "forward" {
                        omission.route.is_some()
                            && omission.route_path.as_deref() == Some(selector.as_str())
                    } else {
                        omission
                            .resolver
                            .as_ref()
                            .is_some_and(|target| target.name == selector)
                    };
                    let mut value = serde_json::to_value(omission).expect("typed omission");
                    value["attribution"] = json!(if associated {
                        if direction == "forward" {
                            "selected_route"
                        } else {
                            "selected_resolver"
                        }
                    } else {
                        "candidate_source"
                    });
                    if associated {
                        selected.push(value);
                    } else {
                        unassociated.push(value);
                    }
                }
            }
            records.sort_by(|a, b| {
                (
                    &a.evidence.source,
                    a.route.start,
                    a.route.end,
                    a.evidence.pair,
                    &a.resolver.source,
                    a.resolver.start,
                    a.resolver.end,
                )
                    .cmp(&(
                        &b.evidence.source,
                        b.route.start,
                        b.route.end,
                        b.evidence.pair,
                        &b.resolver.source,
                        b.resolver.start,
                        b.resolver.end,
                    ))
            });
            sort_omissions(&mut selected);
            sort_omissions(&mut unassociated);
            let mut result = json!({
                "format_version":1, "direction":direction, "count":records.len(),
                "registrations":records, "semantic_omissions":selected,
                "candidate_source_omissions":unassociated,
                "coverage":{
                    "status":"indexed_evidence_only", "fact_semantics":"authoritative_indexed_occurrences",
                    "source_complete":false, "result_semantics":"lower_bound", "omitted_possible":true,
                    "reasons":[{"code":"bounded_written_resolver_registration_scope"}]
                }
            });
            if let Some(discovery) = super::discovery_field(&nomination.report) {
                result["discovery"] = discovery;
            }
            Ok(QueryAnswer::new("resolver_registrations", result))
        }))
    }
    #[cfg(not(feature = "angular"))]
    {
        let _ = (root, selector, scope);
        Err(QueryFailure::invalid("Angular support is unavailable"))
    }
}

#[cfg(feature = "angular")]
fn sort_omissions(values: &mut Vec<Value>) {
    values.sort_by(|a, b| {
        a["source"]
            .as_str()
            .cmp(&b["source"].as_str())
            .then(a["start"].as_u64().cmp(&b["start"].as_u64()))
            .then(a["end"].as_u64().cmp(&b["end"].as_u64()))
            .then(a["reason"].as_str().cmp(&b["reason"].as_str()))
            .then(a["attribution"].as_str().cmp(&b["attribution"].as_str()))
            .then(
                a["route"]["start"]
                    .as_u64()
                    .cmp(&b["route"]["start"].as_u64()),
            )
            .then(
                a["route"]["root"]["start"]
                    .as_u64()
                    .cmp(&b["route"]["root"]["start"].as_u64()),
            )
            .then(
                a["resolver"]["source"]
                    .as_str()
                    .cmp(&b["resolver"]["source"].as_str()),
            )
            .then(
                a["resolver"]["start"]
                    .as_u64()
                    .cmp(&b["resolver"]["start"].as_u64()),
            )
            .then(
                a["resolver"]["end"]
                    .as_u64()
                    .cmp(&b["resolver"]["end"].as_u64()),
            )
    });
    values.dedup();
}
