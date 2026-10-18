use crate::mcp::acquisition::scope_policy_fingerprint;
use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSharpAuthorityManifest {
    identities: Vec<String>,
    scope_policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSharpAuthorityFingerprint(pub(crate) String);

pub(crate) type CSharpDeclarationCatalog = Vec<crate::ir::csharp_exact_calls::Declaration>;

pub(crate) fn authority_manifest(
    artifacts: &[Arc<SemanticCompilationArtifact>],
    roots: impl IntoIterator<Item = String>,
    exclusions: impl IntoIterator<Item = String>,
) -> CSharpAuthorityManifest {
    #[cfg(test)]
    MANIFEST_CONSTRUCTIONS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mut identities: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact.authority_identity_material())
        .collect();
    identities.sort();
    CSharpAuthorityManifest {
        identities,
        scope_policy: scope_policy_fingerprint(roots, exclusions),
    }
}

pub(crate) fn merge_declarations(
    artifacts: &[Arc<SemanticCompilationArtifact>],
) -> CSharpDeclarationCatalog {
    artifacts
        .iter()
        .flat_map(|artifact| {
            artifact
                .csharp_declarations()
                .unwrap_or_default()
                .iter()
                .cloned()
        })
        .collect()
}

pub(crate) fn authority_fingerprint(
    manifest: &CSharpAuthorityManifest,
) -> CSharpAuthorityFingerprint {
    let mut hasher = Sha256::new();
    update(&mut hasher, b"clean-ctx-csharp-authority-v1");
    update(&mut hasher, manifest.scope_policy.as_bytes());
    for identity in &manifest.identities {
        update(&mut hasher, identity.as_bytes());
    }
    CSharpAuthorityFingerprint(format!("{:x}", hasher.finalize()))
}

fn update(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
}

#[cfg(test)]
static MANIFEST_CONSTRUCTIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn reset_manifest_constructions() {
    MANIFEST_CONSTRUCTIONS.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn manifest_constructions() -> usize {
    MANIFEST_CONSTRUCTIONS.load(std::sync::atomic::Ordering::SeqCst)
}
