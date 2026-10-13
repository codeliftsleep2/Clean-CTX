//! Domain-neutral declaration-authority queries.
use super::{EntityKey, WorkspaceIndex, identity_key};
use crate::layers::meta::semantic::EntityRef;

impl WorkspaceIndex {
    pub(super) fn register_declaration(
        &mut self,
        file_path: &str,
        key: &EntityKey,
        entity: EntityRef,
    ) {
        let occurrences = self.declarations.entry(key.clone()).or_default();
        if !occurrences
            .iter()
            .any(|occurrence| occurrence.file == entity.file)
        {
            occurrences.push(entity);
        }
        let keys = self
            .file_declarations
            .entry(file_path.to_string())
            .or_default();
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }

    pub(super) fn remove_declarations_in_file(&mut self, file_path: &str) {
        let Some(keys) = self.file_declarations.remove(file_path) else {
            return;
        };
        for key in keys {
            if let Some(occurrences) = self.declarations.get_mut(&key) {
                occurrences.retain(|entity| entity.file.as_deref() != Some(file_path));
                if occurrences.is_empty() {
                    self.declarations.remove(&key);
                }
            }
        }
    }

    /// Source-proven declaration occurrences matching one semantic identity.
    // Internal architectural oracle pending separate public-query review.
    #[allow(dead_code)]
    pub(crate) fn declarations_by_identity(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
    ) -> Vec<&EntityRef> {
        self.declarations
            .get(&identity_key(domain, entity_type, name))
            .map(|occurrences| occurrences.iter().collect())
            .unwrap_or_default()
    }

    /// Source-proven declarations in one file, separate from relationship endpoints.
    pub(crate) fn declarations_in_file(&self, file_path: &str) -> Vec<&EntityRef> {
        let Some(keys) = self.file_declarations.get(file_path) else {
            return Vec::new();
        };
        let mut declarations = Vec::new();
        for key in keys {
            if let Some(occurrences) = self.declarations.get(key) {
                declarations.extend(
                    occurrences
                        .iter()
                        .filter(|entity| entity.file.as_deref() == Some(file_path)),
                );
            }
        }
        declarations.sort_by(|a, b| {
            (&a.domain, &a.entity_type, &a.name, &a.file)
                .cmp(&(&b.domain, &b.entity_type, &b.name, &b.file))
        });
        declarations
    }
}
