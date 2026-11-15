//! Explicit procedural operator actions. Never exposed as MCP authorization tools.
use super::{governance, profile::Trusted, scan, store, validate};
use crate::pcg::{Result, require};
use rusqlite::{TransactionBehavior, params};
use serde_json::{Value, json};
use std::path::Path;

pub fn describe(profile: &Path) -> Result<Value> {
    inspect(profile, None)
}

/// The CLI/operator must review the exact descriptor before this explicit action.
/// This verifies change binding, not human identity or a hostile shared OS account.
pub fn authorize(profile: &Path, reviewed_digest: &str) -> Result<Value> {
    require(
        reviewed_digest.len() == 64 && reviewed_digest.bytes().all(|b| b.is_ascii_hexdigit()),
        "governance_authorization_digest",
    )?;
    inspect(profile, Some(reviewed_digest))
}

fn inspect(profile: &Path, reviewed_digest: Option<&str>) -> Result<Value> {
    let trusted = Trusted::load(profile)?;
    let mut connection = store::open(&trusted)?;
    let tx = store::sql(connection.transaction_with_behavior(TransactionBehavior::Immediate))?;
    store::initialize(&tx, &trusted)?;
    let mut capture = scan::scan(&trusted)?;
    capture
        .diagnostics
        .extend(validate::validate(&capture.nodes));
    capture
        .diagnostics
        .extend(store::missing(&tx, &capture.nodes)?);
    let prepared = governance::prepare(&tx, &trusted, &mut capture)?;
    let Some(prepared) = prepared else {
        return Ok(
            json!({"authorization_assurance":governance::ASSURANCE,"descriptor":null,
            "diagnostics":capture.diagnostics,"accepted_publication":false}),
        );
    };
    let mut report = json!({"authorization_assurance":governance::ASSURANCE,
        "descriptor":prepared.descriptor,"digest":prepared.digest,"diagnostics":capture.diagnostics,
        "accepted_publication":false,"authorization_recorded":false});
    if let Some(reviewed) = reviewed_digest {
        require(
            reviewed == prepared.digest,
            "governance_authorization_descriptor_mismatch",
        )?;
        require(
            capture.diagnostics.iter().all(|diagnostic| {
                matches!(
                    diagnostic.code.as_str(),
                    "E_GOVERNANCE_BOOTSTRAP_REQUIRED" | "E_GOVERNANCE_AUTHORIZATION_REQUIRED"
                )
            }),
            "governance_candidate_invalid",
        )?;
        require(
            !capture.diagnostics.is_empty(),
            "governance_authorization_not_required",
        )?;
        store::sql(tx.execute(
            "INSERT OR IGNORE INTO document_governance_authorization VALUES(?,?,?)",
            params![
                prepared.digest,
                trusted.profile.workspace_root.to_string_lossy(),
                prepared.predecessor
            ],
        ))?;
        capture.completion(&trusted)?;
        store::sql(tx.commit())?;
        report["authorization_recorded"] = true.into();
    } else {
        capture.completion(&trusted)?;
        // No migration, authorization or graph writes survive a descriptor read.
    }
    Ok(report)
}
