//! Pure conversion from independently validated E5 claims into qualified DTO records.
use super::super::{Interpretation, Prepared, Validation};
use super::{Failure, dto};
use crate::ir::source_observations::{
    WrittenExpression,
    direct_identity::{Evidence, Obligation},
};
use serde_json::{Value, json};
use std::path::PathBuf;

fn obligation(obligation: &Obligation, affected: &str, stage: &str) -> Value {
    use Obligation as O;
    let code = match obligation {
        O::Budget(stop) => dto::stop_code(*stop),
        O::SourceChanged => "source_changed",
        O::UnsupportedExpression => "unsupported_expression",
        O::UnsupportedLexicalScope => "unsupported_lexical_scope",
        O::MissingBinding => "missing_binding",
        O::ConflictingDeclarations => "conflicting_declarations",
        O::TypeOnly => "type_only",
        O::UnsupportedAlias => "unsupported_alias",
        O::UnclosedStarExports => "unclosed_star_exports",
        O::Cycle => "cycle",
        O::UnsupportedSyntax => "unsupported_syntax",
        O::Resolution(_) => "resolution_failed",
        O::ContextualContributorsUnchecked => "contextual_contributors_unchecked",
    };
    dto::issue(code, stage, affected, format!("{obligation:?}"))
}
fn declaration(d: &crate::ir::source_observations::direct_identity::Declaration) -> Value {
    json!({"source":d.source,"source_hash":d.source_hash,"start":d.span.start,"end":d.span.end,"name":d.name,"kind":d.kind})
}
fn attempted(value: &mut Value, id: &str, observation: &str, state: Validation, detail: Value) {
    let mut detail = detail;
    detail["affected"] = json!([id, observation]);
    value["failures"]
        .as_array_mut()
        .expect("array")
        .push(detail);
    value["currentness"]["components"]
        .as_array_mut()
        .expect("array")
        .push(dto::validation(id, state));
}
fn evidence(
    e: &Evidence,
    id: &str,
    observation_id: &str,
    scope: &Value,
    validation: Validation,
    role: bool,
) -> Value {
    let support = e
        .support_content()
        .map(|(p, h)| dto::source(p, h))
        .collect::<Vec<_>>();
    let lookup = |path: &std::path::Path| {
        e.support_content()
            .find(|(p, _)| *p == path)
            .map(|(p, h)| dto::source(p, h))
    };
    let edges=e.edges().iter().filter_map(|edge| Some(json!({"specifier":edge.specifier,"from_source":lookup(&edge.request)?,"to_source":lookup(&edge.target)?,"export_name":if role && edge.specifier=="@angular/router" {Some("Routes")} else {None}}))).collect::<Vec<_>>();
    let packages=e.manifests().iter().filter_map(|m| Some(json!({"manifest":dto::source(&m.path,&m.hash),"name":m.name,"version":m.version.as_ref()?}))).collect::<Vec<_>>();
    let configuration = e
        .support_content()
        .filter(|(p, _)| {
            p.extension().and_then(|x| x.to_str()) == Some("json")
                && p.file_name().and_then(|x| x.to_str()) != Some("package.json")
        })
        .map(|(p, h)| dto::source(p, h))
        .collect::<Vec<_>>();
    let references = e
        .support_content()
        .map(|(p, h)| dto::source_id(p, h))
        .collect::<Vec<_>>();
    json!({"id":id,"kind":if role {"angular_role"}else{"direct_value"},"source_claim":observation_id,"namespace":if e.type_space(){"type"}else{"value"},"target":e.direct_target().map(declaration),"annotation":e.annotation().map(dto::span),"authorized_roots":scope["authorized_roots"],"edges":edges,"packages":packages,"support":support,"configuration":configuration,"validation":dto::validation(id,validation),"obligations":e.obligations().iter().map(|o|obligation(o,id,"direct_support")).collect::<Vec<_>>(),"claim":"direct_declaration_endpoint","classification":if e.direct_target().is_some(){"proven"}else{"unknown"},"claim_scope":scope,"supporting_evidence":references,"applicability":if e.direct_target().is_some(){"supported"}else{"unknown"}})
}
pub(super) fn append(
    value: &mut Value,
    prepared: &Prepared<'_>,
    roots: &[PathBuf],
    source_index: usize,
    record_index: usize,
    errors: &[(&str, &Obligation)],
) -> Result<(), Failure> {
    let records = prepared.records(false);
    let Some(written) = records.written else {
        return Err(Failure::Integrity("missing written record".into()));
    };
    if records.validation.0 != Validation::Current {
        return Err(Failure::Integrity("unvalidated written record".into()));
    }
    let id = format!("observation:{source_index}:{record_index}");
    let source = dto::source(records.source, records.source_hash);
    let scope = json!({"source":source,"object_span":dto::span(written.object),"mapping_span":dto::span(written.mapping_pair),"authorized_roots":roots});
    let expression = match &written.expression {
        WrittenExpression::ReferenceText { text, span } => {
            json!({"text":text,"span":dto::span(*span),"kind":"reference_text","syntax_kind":null})
        }
        WrittenExpression::Unsupported {
            text,
            span,
            syntax_kind,
        } => {
            json!({"text":text,"span":dto::span(*span),"kind":"unsupported","syntax_kind":syntax_kind})
        }
    };
    let observation = json!({"id":id,"source":source,"object":dto::span(written.object),"path_literal":dto::span(written.path_literal),"resolve_pair":dto::span(written.resolve_pair),"mapping_pair":dto::span(written.mapping_pair),"key_span":dto::span(written.key_span),"route_path":written.route_path,"key":written.key,"expression":expression,"classification":"proven","claim":"written_source_mapping","validation":dto::validation(&id,records.validation.0),"claim_scope":scope,"supporting_evidence":[dto::source_id(records.source,records.source_hash)],"applicability":"supported"});
    value["observations"]
        .as_array_mut()
        .expect("array")
        .push(observation);
    value["currentness"]["components"]
        .as_array_mut()
        .expect("array")
        .push(dto::validation(&id, Validation::Current));
    let mut support = vec![id.clone()];
    let mut value_id = None;
    let mut role_id = None;
    for (item, current, role) in [
        (prepared.direct_evidence(), records.validation.1, false),
        (
            prepared.role_evidence().map(|r| r.evidence()),
            records.validation.2,
            true,
        ),
    ] {
        let Some(item) = item else {
            continue;
        };
        if current != Validation::Current {
            let attempt = format!(
                "attempt:{}:{source_index}:{record_index}",
                if role { "role" } else { "direct" }
            );
            attempted(
                value,
                &attempt,
                &id,
                current,
                dto::issue(
                    "unvalidated",
                    "serving_boundary",
                    &attempt,
                    format!(
                        "{} evidence: {current:?}",
                        if role { "role" } else { "direct" }
                    ),
                ),
            );
            continue;
        }
        let evidence_id = format!(
            "{}:{source_index}:{record_index}",
            if role { "role" } else { "direct" }
        );
        for problem in item
            .obligations()
            .iter()
            .filter(|o| **o != Obligation::ContextualContributorsUnchecked)
        {
            let mut issue = obligation(
                problem,
                &evidence_id,
                if role {
                    "angular_role"
                } else {
                    "direct_support"
                },
            );
            issue["affected"]
                .as_array_mut()
                .expect("array")
                .push(json!(id));
            value["failures"].as_array_mut().expect("array").push(issue);
        }
        value["direct_evidence"]
            .as_array_mut()
            .expect("array")
            .push(evidence(item, &evidence_id, &id, &scope, current, role));
        value["currentness"]["components"]
            .as_array_mut()
            .expect("array")
            .push(dto::validation(&evidence_id, current));
        support.push(evidence_id.clone());
        if role {
            role_id = Some(evidence_id);
        } else {
            value_id = Some(evidence_id);
        }
    }
    let mut obligations = records
        .direct_obligations
        .iter()
        .chain(records.role_obligations)
        .map(|o| obligation(o, &id, "interpretation"))
        .collect::<Vec<_>>();
    for (stage, error) in errors {
        let issue = obligation(error, &id, stage);
        obligations.push(issue.clone());
        let attempt = format!(
            "attempt:{}:{source_index}:{record_index}",
            if *stage == "angular_role" {
                "role"
            } else {
                "direct"
            }
        );
        let state = if let Obligation::Budget(stop) = error {
            Validation::Unvalidated(*stop)
        } else {
            Validation::Missing
        };
        attempted(value, &attempt, &id, state, issue);
    }
    if value_id.is_none() || role_id.is_none() {
        obligations.push(dto::issue(
            "unresolved_identity",
            "interpretation",
            &id,
            "one or more identity/role components not admitted",
        ));
    }
    let strength = match records.classification.interpretation {
        Interpretation::StronglySupported => "strongly_supported",
        Interpretation::Partial | Interpretation::Unsupported => "partial",
        Interpretation::Unknown => "unknown",
    };
    value["interpretations"].as_array_mut().expect("array").push(json!({"id":format!("interpretation:{source_index}:{record_index}"),"source_claim":id,"value_evidence":value_id,"role_evidence":role_id,"classification":strength,"claim":"written_angular_routes_mapping","composition_compatible":records.classification.composition_compatible,"obligations":obligations,"not_established":["runtime_execution","canonical_registration","complete_workspace_coverage","javascript_implementation","contextual_closure"],"claim_scope":scope,"supporting_evidence":support,"applicability":if records.classification.interpretation==Interpretation::Unsupported{"unsupported"}else if strength=="strongly_supported"{"supported"}else{"unknown"}}));
    Ok(())
}
