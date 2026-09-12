// Project-explicit GraphBridge search used by bounded workspace hydration.

use super::bridge::{GraphBridge, GraphNode, map_search_result};
use super::client::CbmError;
use std::path::PathBuf;

fn escape_cypher_string_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn inbound_reference_query(target_name: &str) -> String {
    let escaped = escape_cypher_string_literal(target_name);
    format!(
        "MATCH (caller)-[r]->(target {{name: '{escaped}'}}) \
         WHERE type(r) <> 'DEFINES' AND type(r) <> 'DEFINES_METHOD' \
         RETURN caller.file_path"
    )
}

impl GraphBridge {
    /// Return every configured CBM project with its canonical repository root.
    /// The order is deterministic and independent of the active project.
    pub(crate) fn configured_projects(&self) -> Vec<(String, PathBuf)> {
        let mut projects: Vec<_> = self
            .project_paths
            .iter()
            .map(|(project, root)| (project.clone(), root.clone()))
            .collect();
        projects.sort_by(|left, right| left.0.cmp(&right.0));
        projects
    }

    /// Search one explicit project without changing persistent active-project state.
    pub(crate) fn search_in_project(
        &mut self,
        project: &str,
        query: &str,
    ) -> Result<Vec<GraphNode>, CbmError> {
        let query = query.to_string();
        let has_regex = query.chars().any(|character| {
            matches!(
                character,
                '.' | '*' | '+' | '[' | '(' | '\\' | '^' | '$' | '{' | '|'
            )
        });
        let name_pattern = if has_regex {
            query
        } else {
            format!(".*{query}.*")
        };

        let result = {
            let mut guard = self
                .client
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let client = guard
                .as_mut()
                .ok_or_else(|| CbmError::LaunchError("CBM not available".into()))?;
            client.search_graph(&name_pattern, project, None)
        };
        self.update_status();
        let nodes = result?;
        Ok(nodes.iter().filter_map(map_search_result).collect())
    }

    /// Return files that may contain inbound references to `target_name` in
    /// one explicit project, without changing persistent active-project state.
    ///
    /// Only projected path strings cross this boundary. Relationship type,
    /// direction, cardinality, and graph identity remain advisory CBM data.
    pub(crate) fn inbound_reference_paths_in_project(
        &mut self,
        project: &str,
        target_name: &str,
    ) -> Result<Vec<String>, CbmError> {
        let query = inbound_reference_query(target_name);
        let result = {
            let mut guard = self
                .client
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let client = guard
                .as_mut()
                .ok_or_else(|| CbmError::LaunchError("CBM not available".into()))?;
            client.query_graph(&query, project)
        };
        self.update_status();
        let table = result?;
        Ok(table
            .rows
            .iter()
            .filter_map(|row| {
                row.first()
                    .and_then(|value| value.as_str())
                    .map(String::from)
            })
            .filter(|path| !path.is_empty())
            .collect())
    }
}

#[cfg(test)]
#[path = "../tests/cbm/project_search.rs"]
mod tests;
