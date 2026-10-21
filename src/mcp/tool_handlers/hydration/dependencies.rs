//! Structured dependency acquisition through the shared proof and cache boundary.
use super::HydrationRequirement;
use super::acquisition::acquisition_cache_key;
use super::filesystem::{configured_roots, root_key, source_inventory};
use super::publication::{CandidatePublication, compile_candidates_for};
use crate::ir::csharp_resolution::{CSharpCompilationUnit, ancestry_obligations};
use crate::mcp::McpState;
use crate::mcp::acquisition::*;
use crate::mcp::csharp_calls::csharp_resolution_partition;
use crate::mcp::discovery_cache::DiscoveryMode;
use crate::mcp::hydration_flight::HydrationFlightKey;
use crate::workspace::index::{SemanticFidelity, WorkspaceIndex};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(crate) struct DependencyEvidence {
    outcome: AcquisitionOutcome,
    reasons: BTreeSet<&'static str>,
    generation: u64,
}

impl DependencyEvidence {
    pub(crate) fn publication_stamp(
        &self,
    ) -> crate::mcp::semantic_compilation::dependencies::PublicationStamp {
        crate::mcp::semantic_compilation::dependencies::PublicationStamp::for_generation(
            self.generation,
        )
    }
    pub(crate) fn publication_is_current(&self, index: &WorkspaceIndex) -> bool {
        index.has_csharp_publication_generation(self.generation)
    }
    /// Evaluation checks publication again: neither a flight wake-up nor a
    /// preparation-time certificate authorizes a later, incompatible index view.
    pub(crate) fn apply(&self, index: &WorkspaceIndex, coverage: &mut Value) {
        let current = index.has_csharp_publication_generation(self.generation);
        let mut reasons = self.reasons.clone();
        if !current {
            reasons.insert("dependency_publication_changed");
        }
        if self.outcome.satisfaction().is_source_complete() && current {
            if coverage["dependency_proof"]["status"] == "unproven" {
                return;
            }
            coverage["dependency_proof"] = json!({
                "status":"complete",
                "generation":self.outcome.satisfaction().completeness_generation(),
            });
            if let Some(requirements) = coverage
                .get_mut("evidence_requirements")
                .and_then(Value::as_array_mut)
            {
                let requirement = json!("dependencies_current");
                if !requirements.contains(&requirement) {
                    requirements.push(requirement);
                }
            }
        } else {
            if let Some(existing) = coverage["reasons"].as_array() {
                // Existing producer omission reasons remain separate evidence.
                let existing = existing.clone();
                coverage["reasons"] = json!(
                    existing
                        .into_iter()
                        .chain(reasons.iter().map(|reason| json!(reason)))
                        .collect::<Vec<_>>()
                );
            } else {
                coverage["reasons"] = json!(reasons);
            }
            coverage["status"] = json!("semantic_dependencies_unresolved");
            coverage["source_complete"] = json!(false);
            coverage["omitted_possible"] = json!(true);
            coverage["result_semantics"] = json!("lower_bound");
            coverage["dependency_proof"] = json!({"status":"unproven"});
            if let Some(requirements) = coverage
                .get_mut("evidence_requirements")
                .and_then(Value::as_array_mut)
            {
                requirements.retain(|item| item != "dependencies_current");
            }
        }
    }
}

pub(crate) fn acquire_csharp_dependencies(
    state: &McpState,
    owner: &str,
    workspace_root: Option<&str>,
) -> Result<DependencyEvidence, String> {
    let roots = configured_roots(state, workspace_root);
    // Flight identity excludes mutable publication state. Followers rerun attempt
    // and rebuild their cache key from the current authoritative generation.
    let target = target(owner.to_string());
    let flight = HydrationFlightKey::new(
        roots.iter().map(|root| root_key(root)).collect(),
        DiscoveryMode::Declaration,
        owner,
        Some(SemanticFidelity::High),
        Some(target),
    );
    state
        .hydration_flights
        .run(flight, || attempt(state, owner, workspace_root))
}

fn target(identity: String) -> AcquisitionTarget {
    AcquisitionTarget {
        capability: ProducerCapability::SourceDependencyClosure,
        domain: "csharp".into(),
        entity_type: "ancestry".into(),
        written_name: identity,
        matching_version: 2,
    }
}

fn cache_key(
    state: &McpState,
    owner: &str,
    roots: Vec<String>,
) -> Result<AcquisitionCacheKey, String> {
    let source = state
        .read_source(owner)
        .map_err(|error| error.to_string())?;
    let identity = serde_json::to_string(&(
        owner,
        csharp_resolution_partition(state, owner),
        crate::cache::content_hash(source.as_bytes()),
        state.csharp_authority_generation(),
    ))
    .map_err(|error| error.to_string())?;
    let mut key = acquisition_cache_key(target(identity), roots, &state.config.exclude_patterns);
    key.predicate_version = 1;
    key.proof_contract_version = 1;
    Ok(key)
}

fn attempt(
    state: &McpState,
    owner: &str,
    workspace_root: Option<&str>,
) -> Result<DependencyEvidence, String> {
    loop {
        let evidence = attempt_once(state, owner, workspace_root)?;
        // Frontier publication can discover a changed cached source and retire
        // the root observation captured by inventory. Rebuild the whole proof
        // from the surviving observation rather than returning that superseded
        // request-local inventory as an unresolved semantic dependency.
        if !evidence
            .reasons
            .contains("dependency_source_generation_changed")
        {
            return Ok(evidence);
        }
    }
}

fn attempt_once(
    state: &McpState,
    owner: &str,
    workspace_root: Option<&str>,
) -> Result<DependencyEvidence, String> {
    let roots = configured_roots(state, workspace_root);
    let root_names: Vec<_> = roots.iter().map(|root| root_key(root)).collect();
    let requested_generation = state.csharp_authority_generation();
    let key = cache_key(state, owner, root_names.clone())?;
    let cached = state.acquisition_completion_lock().get(&key);
    let current_generation = state.csharp_authority_generation();
    let artifact = state.csharp_call_authority.artifact(owner);
    let cached_projection_current = {
        let index = state.workspace_index_read();
        index.has_csharp_publication_generation(current_generation)
            && artifact.as_ref().is_some_and(|artifact| {
                index.has_current_semantic_projection(
                    owner,
                    SemanticFidelity::High,
                    artifact.source_hash(),
                )
            })
    };
    if let Some(satisfaction) = cached
        && requested_generation == current_generation
        && cached_projection_current
        && let Some(outcome) = AcquisitionOutcome::from_cached_complete(satisfaction)
    {
        return Ok(DependencyEvidence {
            outcome,
            reasons: BTreeSet::new(),
            generation: current_generation,
        });
    }
    let (inventory, root_generation) = loop {
        let before = state
            .acquisition_completion_lock()
            .root_generation_snapshot(&root_names);
        let inventory = source_inventory(state, &roots);
        if before
            == state
                .acquisition_completion_lock()
                .root_generation_snapshot(&root_names)
        {
            break (inventory, before);
        }
    };
    let partition = csharp_resolution_partition(state, owner);
    let entries: Vec<_> = inventory
        .entries
        .iter()
        .filter(|entry| {
            entry.source_kind == SourceKind::CSharp
                && csharp_resolution_partition(state, &entry.owner) == partition
        })
        .collect();
    let mut selected = BTreeSet::from([owner.to_string()]);
    let mut processed = BTreeSet::new();
    let external = crate::dotnet_meta::semantic::known_csharp_type_identities();
    let mut processing_failed = false;
    let obligations = loop {
        let frontier: Vec<_> = selected.difference(&processed).cloned().collect();
        if !publish(state, &frontier, workspace_root)? {
            processing_failed = true;
        }
        processed.extend(frontier);
        let units: Vec<CSharpCompilationUnit> = selected
            .iter()
            .filter_map(|file| {
                state
                    .csharp_call_authority
                    .artifact(file)
                    .and_then(|artifact| artifact.csharp_resolution_unit().cloned())
            })
            .collect();
        let obligations = ancestry_obligations(&units, owner, &external);
        let before = selected.len();
        selected.extend(
            entries
                .iter()
                .filter(|entry| {
                    obligations
                        .discovery_terms
                        .iter()
                        .any(|term| entry.source.contains(term))
                })
                .map(|entry| entry.owner.clone()),
        );
        if selected.len() == before {
            break obligations;
        }
    };
    let obligation_identity: Vec<_> = obligations
        .requirements
        .iter()
        .map(|requirement| requirement.identity_material())
        .collect();
    let evidence_generation = crate::cache::content_hash(
        format!("{}:{obligation_identity:?}", inventory.digest).as_bytes(),
    );
    let mut reasons = obligations.reasons;
    if state
        .acquisition_completion_lock()
        .root_generation_snapshot(&root_names)
        != root_generation
    {
        reasons.insert("dependency_source_generation_changed");
    }
    if !entries.iter().any(|entry| entry.owner == owner) {
        reasons.insert("dependency_scope_unavailable");
    }
    if !obligations.owners.is_subset(&selected) {
        reasons.insert("dependency_unresolved");
    }
    if !inventory.complete {
        reasons.insert("dependency_inventory_incomplete");
    }
    if processing_failed {
        reasons.insert("dependency_processing_failed");
    }
    if state.csharp_call_authority.artifact(owner).is_none() {
        reasons.insert("dependency_processing_failed");
    }
    let generation = state.prepare_exact_csharp_calls()?;
    // Bind satisfaction to the final, reconciled generation, not the generation
    // before publishing the frontier. Refresh invalidates the shared root cache.
    let key = cache_key(state, owner, root_names.clone())?;
    let producer = key
        .producer_generations
        .iter()
        .find(|(kind, _)| *kind == SourceKind::CSharp)
        .map(|(_, generation)| generation.clone())
        .ok_or("Missing C# producer generation")?;
    let units: Vec<_> = root_names
        .iter()
        .map(|root| CoverageUnit {
            canonical_root: root.clone(),
            source_kind: SourceKind::CSharp,
            producer: ProducerCapability::SourceDependencyClosure,
            producer_generation: producer.clone(),
            relationships: BTreeSet::new(),
        })
        .collect();
    let scope = scope_policy_fingerprint(root_names, state.config.exclude_patterns.clone());
    let requirement =
        AcquisitionRequirement::for_target(key.target.clone(), scope.clone(), units.clone());
    let mut satisfaction = AcquisitionSatisfaction::new(&requirement);
    let mut dispositions = BTreeMap::new();
    let publication_current = selected.iter().all(|file| {
        state
            .csharp_call_authority
            .artifact(file)
            .is_some_and(|artifact| {
                state.read_source(file).is_ok_and(|source| {
                    let hash = crate::cache::content_hash(source.as_bytes());
                    artifact.source_hash() == hash
                        && state
                            .workspace_index_read()
                            .has_current_semantic_projection(file, SemanticFidelity::High, &hash)
                })
            })
    }) && generation == state.csharp_authority_generation();
    if !publication_current {
        reasons.insert("dependency_publication_changed");
    }
    for unit in units {
        if reasons.is_empty() {
            let claim = AcquisitionProofClaim {
                unit: unit.clone(),
                target: key.target.clone(),
                scope_fingerprint: scope.clone(),
                inventory_generation: evidence_generation.clone(),
                admission_fingerprint: scope.clone(),
                predicate_version: 1,
                proof_contract_version: 1,
                exhaustive: true,
                truncated: false,
                limitations: Vec::new(),
                candidate_owners: selected
                    .iter()
                    .filter(|file| std::path::Path::new(file).starts_with(&unit.canonical_root))
                    .cloned()
                    .collect(),
                provenance: ProofProvenance::Clean,
            };
            let context = ProofContext {
                inventory_generation: evidence_generation.clone(),
                admission_fingerprint: scope.clone(),
                predicate_version: 1,
                proof_contract_version: 1,
            };
            let proof = validate_proof_claim(&requirement, &context, claim)
                .map_err(|error| format!("Dependency proof rejected: {error:?}"))?;
            satisfaction.record_after_publication(proof, SemanticPublicationStatus::Complete);
            dispositions.insert(unit, CoverageProofDisposition::Proven);
        } else {
            let barrier = if !inventory.complete {
                ProofBarrier::InventoryIncomplete
            } else if processing_failed {
                ProofBarrier::CandidateProcessingFailed
            } else if !publication_current {
                ProofBarrier::PublicationFailed
            } else {
                ProofBarrier::DependencyUnresolved
            };
            dispositions.insert(unit, CoverageProofDisposition::Unproven(barrier));
        }
    }
    if satisfaction.is_source_complete() {
        state
            .acquisition_completion_lock()
            .commit(key, satisfaction.clone());
    }
    let outcome = AcquisitionOutcome::new(
        satisfaction,
        dispositions,
        if publication_current {
            SemanticPublicationStatus::Complete
        } else {
            SemanticPublicationStatus::Failed
        },
    );
    Ok(DependencyEvidence {
        outcome,
        reasons,
        generation,
    })
}

pub(in crate::mcp::tool_handlers) fn publish(
    state: &McpState,
    frontier: &[String],
    root: Option<&str>,
) -> Result<bool, String> {
    let pending: Vec<_> = frontier
        .iter()
        .filter(|file| {
            let Some(artifact) = state.csharp_call_authority.artifact(file) else {
                return true;
            };
            !state.read_source(file).is_ok_and(|source| {
                let hash = crate::cache::content_hash(source.as_bytes());
                artifact.source_hash() == hash
                    && state
                        .workspace_index_read()
                        .semantic_projection_fidelity(file)
                        .is_some_and(|fidelity| fidelity >= SemanticFidelity::High)
            })
        })
        .cloned()
        .collect();
    Ok(compile_candidates_for(
        state,
        &pending,
        root,
        HydrationRequirement::Semantic(SemanticFidelity::High),
    )?
    .iter()
    .all(|outcome| {
        matches!(
            outcome,
            CandidatePublication::Published
                | CandidatePublication::PublishedCSharp
                | CandidatePublication::Current
        )
    }))
}
