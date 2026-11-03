//! Ordered Vec storage stays authoritative. These paired mutations maintain
//! exact membership companions, never semantic/currentness authority.
use super::{EntityKey, WorkspaceIndex};
use crate::layers::meta::semantic::EntityRef;
impl WorkspaceIndex {
    /// Register a single entity occurrence.
    ///
    /// Occurrence identity is (domain, entity_type, name, file): registering
    /// the same occurrence again is a no-op. An entity that participates in
    /// many edges within one file is registered exactly once for that file;
    /// the same identity in a different file remains a distinct occurrence
    /// (cross-file ambiguity preserved).
    pub(super) fn register_entity(
        &mut self,
        key: &EntityKey,
        entity: EntityRef,
        file_entity_keys: &mut Vec<EntityKey>,
    ) {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|s| s.membership_probes[0] += 1);
        let members = self.name_members.entry(entity.name.clone()).or_default();
        if !members.contains(key) {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|s| s.membership_probes[0] += 1);
            members.insert(key.clone());
            self.name_index
                .entry(entity.name.clone())
                .or_default()
                .push(key.clone());
        }
        // Publisher participation is independent of physical occurrence dedup.
        file_entity_keys.push(key.clone());
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|s| s.membership_probes[1] += 1);
        let members = self.occurrence_members.entry(key.clone()).or_default();
        if !members.contains(&entity.file) {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|s| s.membership_probes[1] += 1);
            members.insert(entity.file.clone());
            self.entities.entry(key.clone()).or_default().push(entity);
        }
    }
    pub(super) fn register_publisher_key(&mut self, file: &str, key: EntityKey) {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|s| s.membership_probes[2] += 1);
        let members = self.publisher_members.entry(file.to_owned()).or_default();
        if !members.contains(&key) {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|s| s.membership_probes[2] += 1);
            self.file_map
                .entry(file.to_owned())
                .or_default()
                .push(key.clone());
            members.insert(key);
        }
    }
    pub(super) fn has_physical_occurrence(&self, key: &EntityKey, file: &Option<String>) -> bool {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|s| s.membership_probes[1] += 1);
        self.occurrence_members
            .get(key)
            .is_some_and(|members| members.contains(file))
    }
    pub(super) fn take_publisher_keys(&mut self, file: &str) -> Option<Vec<EntityKey>> {
        self.publisher_members.remove(file);
        self.file_map.remove(file)
    }
    pub(super) fn remove_physical_occurrence(&mut self, key: &EntityKey, file: &str) {
        if let Some(items) = self.entities.get_mut(key) {
            items.retain(|item| item.file.as_deref() != Some(file));
            if items.is_empty() {
                self.entities.remove(key);
                self.occurrence_members.remove(key);
            } else if let Some(members) = self.occurrence_members.get_mut(key) {
                members.retain(|item| item.as_deref() != Some(file));
            }
        }
    }
    pub(super) fn remove_publisher_key(&mut self, file: &str, key: &EntityKey) {
        if let Some(keys) = self.file_map.get_mut(file) {
            keys.retain(|item| item != key);
        }
        if let Some(members) = self.publisher_members.get_mut(file) {
            members.remove(key);
        }
    }
    pub(super) fn clean_name_membership(&mut self, name: &str) {
        if let Some(keys) = self.name_index.get_mut(name) {
            keys.retain(|key| self.entities.contains_key(key));
            if keys.is_empty() {
                self.name_index.remove(name);
                self.name_members.remove(name);
            } else if let Some(members) = self.name_members.get_mut(name) {
                members.retain(|key| self.entities.contains_key(key));
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/workspace/index_membership_lifecycle_af_perf_002.rs"]
mod lifecycle_tests;
