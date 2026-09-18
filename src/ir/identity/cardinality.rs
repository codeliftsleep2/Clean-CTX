use super::{IdentityError, IdentityKind};
use std::collections::HashMap;
use std::hash::Hash;

pub(super) fn insert_definition<K: Eq + Hash>(
    definitions: &mut HashMap<K, usize>,
    key: K,
    operation: &'static str,
    kind: IdentityKind,
    raw_id: &str,
    owner: Option<&str>,
    instruction: usize,
) -> Result<(), IdentityError> {
    if let Some(first_instruction) = definitions.insert(key, instruction) {
        return Err(IdentityError::DuplicateIdentity {
            operation,
            kind,
            id: raw_id.to_owned(),
            owner: owner.map(str::to_owned),
            first_instruction,
            duplicate_instruction: instruction,
        });
    }
    Ok(())
}

pub(super) fn insert_singular<K: Eq + Hash>(
    facts: &mut HashMap<K, usize>,
    key: K,
    operation: &'static str,
    target_kind: IdentityKind,
    target_id: &str,
    instruction: usize,
) -> Result<(), IdentityError> {
    if let Some(first_instruction) = facts.insert(key, instruction) {
        return Err(IdentityError::DuplicateFact {
            operation,
            target_kind,
            target_id: target_id.to_owned(),
            first_instruction,
            duplicate_instruction: instruction,
        });
    }
    Ok(())
}
