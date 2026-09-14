//! Stable normalization for reconstructed Reactive Forms artifacts.

use super::{FieldDecl, FieldKind, FormArtifact};

pub(super) fn normalize_artifacts(artifacts: &mut Vec<FormArtifact>) {
    let mut normalized: Vec<FormArtifact> = Vec::new();
    for artifact in artifacts.drain(..) {
        let compatible = normalized
            .iter()
            .position(|existing| match (existing, &artifact) {
                (FormArtifact::Form(left), FormArtifact::Form(right)) => left.name == right.name,
                (FormArtifact::Field(left), FormArtifact::Field(right)) => {
                    left.name == right.name && same_field_kind(&left.kind, &right.kind)
                }
                _ => false,
            });
        if let Some(index) = compatible {
            merge_artifact(&mut normalized[index], artifact);
        } else {
            normalized.push(artifact);
        }
    }
    *artifacts = normalized;
}

fn merge_artifact(existing: &mut FormArtifact, incoming: FormArtifact) {
    match (existing, incoming) {
        (FormArtifact::Form(left), FormArtifact::Form(right)) => {
            merge_fields(&mut left.fields, right.fields);
        }
        (FormArtifact::Field(left), FormArtifact::Field(right)) => merge_field(left, right),
        _ => unreachable!("normalization merges only compatible artifacts"),
    }
}

pub(super) fn merge_fields(existing: &mut Vec<FieldDecl>, incoming: Vec<FieldDecl>) {
    for field in incoming {
        if let Some(current) = existing.iter_mut().find(|current| {
            current.name == field.name && same_field_kind(&current.kind, &field.kind)
        }) {
            merge_field(current, field);
        } else {
            // Preserve incompatible evidence rather than imposing kind precedence.
            existing.push(field);
        }
    }
}

fn merge_field(existing: &mut FieldDecl, incoming: FieldDecl) {
    merge_unique(&mut existing.validators, incoming.validators);
    match (&mut existing.kind, incoming.kind) {
        (FieldKind::Group(left), FieldKind::Group(right))
        | (FieldKind::Array(left), FieldKind::Array(right)) => merge_fields(left, right),
        (FieldKind::Control, FieldKind::Control) => {}
        _ => unreachable!("field merge requires matching structural kinds"),
    }
}

fn same_field_kind(left: &FieldKind, right: &FieldKind) -> bool {
    matches!(
        (left, right),
        (FieldKind::Control, FieldKind::Control)
            | (FieldKind::Group(_), FieldKind::Group(_))
            | (FieldKind::Array(_), FieldKind::Array(_))
    )
}

fn merge_unique(existing: &mut Vec<String>, incoming: Vec<String>) {
    for value in incoming {
        if !existing.contains(&value) {
            existing.push(value);
        }
    }
}
