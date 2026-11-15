//! Accepted governance and exact procedural authorization, within the graph transaction.
use super::{
    governance_state, membership, model::Diagnostic, profile::Trusted, scan::Capture, store,
};
use crate::pcg::{Error, Result, identity, require};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

pub(super) const ASSURANCE: &str = "procedural_operator_action_not_authenticated_identity";

pub(super) struct Prepared {
    pub state: Value,
    pub state_digest: String,
    pub descriptor: Value,
    pub digest: String,
    pub predecessor: Option<String>,
    pub authorization: Option<String>,
}

pub(super) fn prepare(
    c: &Connection,
    trusted: &Trusted,
    capture: &mut Capture,
) -> Result<Option<Prepared>> {
    if capture
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code != "E_DECLARATION_REMOVED")
    {
        return Ok(None);
    }
    let predecessor = store::head(c)?;
    let accepted: Option<(String, String)> = store::sql(c.query_row(
        "SELECT state,state_digest FROM document_governance_publication WHERE g=(SELECT g FROM document_head WHERE slot=1)",
        [], |row| Ok((row.get(0)?, row.get(1)?))).optional())?;
    let protected_exists = trusted
        .profile
        .workspace_root
        .join(governance_state::PROTECTED)
        .try_exists()
        .map_err(|_| Error("document_source_unavailable"))?;
    if accepted.is_none() && !protected_exists {
        return Ok(None);
    }
    let corpus_exists = trusted
        .profile
        .workspace_root
        .join(membership::REGISTRY)
        .try_exists()
        .map_err(|_| Error("document_source_unavailable"))?;
    if !corpus_exists || !protected_exists {
        let mut diagnostic = Diagnostic::new(
            "E_ACCEPTED_GOVERNANCE_REMOVED",
            membership::REGISTRY,
            None,
            "Restore both accepted governance registries; candidate omission cannot discharge accepted obligations",
        );
        diagnostic.admission_disposition = Some("rejected");
        diagnostic.accepted_publication = Some(false);
        capture.diagnostics.push(diagnostic);
        return Ok(None);
    }
    let state = match governance_state::read(capture, trusted) {
        Ok(state) => state,
        Err(error) => {
            let mut diagnostic = Diagnostic::new(
                "E_GOVERNANCE_STATE",
                governance_state::PROTECTED,
                None,
                error.0,
            );
            diagnostic.admission_disposition = Some("rejected");
            diagnostic.accepted_publication = Some(false);
            capture.diagnostics.push(diagnostic);
            return Ok(None);
        }
    };
    let state_digest = identity::hash(&state);
    let old_state = accepted
        .as_ref()
        .map(|(text, digest)| -> Result<Value> {
            let value: Value =
                serde_json::from_str(text).map_err(|_| Error("governance_corruption"))?;
            require(identity::hash(&value) == *digest, "governance_corruption")?;
            Ok(value)
        })
        .transpose()?;
    if let Some(old) = &old_state {
        // An explicitly proposed removal from accepted membership is a governed
        // change, not an unexplained disappearance. Defer only that legacy error
        // to exact authorization below; every other admission error remains fatal.
        let identities = |membership: &Value| -> Result<std::collections::BTreeSet<String>> {
            let mut ids = std::collections::BTreeSet::new();
            for values in membership
                .as_object()
                .ok_or(Error("governance_corruption"))?
                .values()
            {
                for id in values.as_array().ok_or(Error("governance_corruption"))? {
                    ids.insert(
                        id.as_str()
                            .ok_or(Error("governance_corruption"))?
                            .to_owned(),
                    );
                }
            }
            Ok(ids)
        };
        let previous_ids = identities(&old["membership"])?;
        let proposed_ids = identities(&state["membership"])?;
        capture.diagnostics.retain(|diagnostic| {
            diagnostic.code != "E_DECLARATION_REMOVED"
                || !diagnostic
                    .node
                    .as_ref()
                    .is_some_and(|id| previous_ids.contains(id) && !proposed_ids.contains(id))
        });
    }
    if !capture.diagnostics.is_empty() {
        return Ok(None);
    }
    let previous_view = if let Some(old) = &old_state {
        governance_state::project(
            &capture.nodes,
            old["protection"]
                .as_array()
                .ok_or(Error("governance_corruption"))?,
            true,
        )?
    } else {
        Value::Null
    };
    let descriptor = json!({"version":1,"workspace":trusted.profile.workspace_root,
        "catalog_scope":trusted.scope_fingerprint,"predecessor_generation":predecessor,
        "accepted_membership_digest":old_state.as_ref().map(|old|identity::hash(&old["membership"])),
        "accepted_protection_digest":old_state.as_ref().map(|old|identity::hash(&old["protection"])),
        "accepted_state_digest":accepted.as_ref().map(|(_,digest)|digest),
        "proposed_state":state,"proposed_state_digest":state_digest,
        "previously_protected_candidate_view":previous_view});
    let digest = identity::hash(&descriptor);
    let changed = accepted
        .as_ref()
        .is_none_or(|(_, old)| old != &state_digest);
    let authorization: Option<(String, Option<String>)> = store::sql(
        c.query_row(
            "SELECT workspace,predecessor_g FROM document_governance_authorization WHERE digest=?",
            [&digest],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional(),
    )?;
    let authorized = authorization.as_ref().is_some_and(|(workspace, previous)| {
        workspace.as_str() == trusted.profile.workspace_root.to_string_lossy().as_ref()
            && previous == &predecessor
    });
    if changed && !authorized {
        let mut diagnostic = Diagnostic::new(
            if accepted.is_none() {
                "E_GOVERNANCE_BOOTSTRAP_REQUIRED"
            } else {
                "E_GOVERNANCE_AUTHORIZATION_REQUIRED"
            },
            membership::REGISTRY,
            None,
            "Present this exact descriptor for explicit procedural operator authorization; agents must not submit authorization",
        );
        diagnostic.change_descriptor = Some(descriptor.clone());
        diagnostic.authorization_digest = Some(digest.clone());
        diagnostic.authorization_assurance = Some(ASSURANCE);
        diagnostic.admission_disposition = Some("rejected");
        diagnostic.accepted_publication = Some(false);
        capture.diagnostics.push(diagnostic);
    }
    let authorization = (changed && authorized).then(|| digest.clone());
    Ok(Some(Prepared {
        state,
        state_digest,
        descriptor,
        digest,
        predecessor,
        authorization,
    }))
}

pub(super) fn publish(c: &Connection, generation: &str, prepared: &Prepared) -> Result<()> {
    store::sql(c.execute(
        "INSERT OR IGNORE INTO document_governance_publication VALUES(?,?,?,?,?)",
        params![
            generation,
            identity::canonical(&prepared.state),
            prepared.state_digest,
            prepared.authorization,
            prepared.predecessor
        ],
    ))?;
    let stored: (String, String) = store::sql(c.query_row(
        "SELECT state,state_digest FROM document_governance_publication WHERE g=?",
        [generation],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ))?;
    require(
        stored
            == (
                identity::canonical(&prepared.state),
                prepared.state_digest.clone(),
            ),
        "governance_corruption",
    )
}
