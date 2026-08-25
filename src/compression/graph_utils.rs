// src/compression/graph_utils.rs
//
// Shared graph algorithms for meta-layer graphs.
//
// This module provides common graph algorithms (cycle detection, transitive
// dependencies) that are used across Angular, Spring, and .NET meta-layer
// graphs. Extracting these to a shared location eliminates code duplication
// and ensures consistent behavior across frameworks.

use std::collections::{HashMap, VecDeque};

/// Check if a directed graph contains a cycle using three-color DFS.
///
/// # Arguments
/// * `node_count` - Total number of nodes in the graph
/// * `adjacency_fn` - Function that returns neighbors for a given node ID
///
/// # Returns
/// `true` if at least one cycle is detected, `false` otherwise.
///
/// # Complexity
/// O(V + E) time, O(V) space
pub fn has_cycle<F>(node_count: usize, adjacency_fn: F) -> bool
where
    F: Fn(usize) -> Vec<usize>,
{
    if node_count == 0 {
        return false;
    }

    let mut color: HashMap<usize, u8> = HashMap::with_capacity(node_count);
    for i in 0..node_count {
        color.insert(i, 0);
    }

    fn dfs<F>(node: usize, adj_fn: &F, color: &mut HashMap<usize, u8>) -> bool
    where
        F: Fn(usize) -> Vec<usize>,
    {
        color.insert(node, 1);
        let neighbors = adj_fn(node);
        for next in neighbors {
            match color.get(&next).copied().unwrap_or(0) {
                1 => return true,
                0 if dfs(next, adj_fn, color) => return true,
                _ => {}
            }
        }
        color.insert(node, 2);
        false
    }

    for node in 0..node_count {
        if color.get(&node).copied().unwrap_or(0) == 0 && dfs(node, &adjacency_fn, &mut color) {
            return true;
        }
    }

    false
}

/// Find all cycles in a directed graph using DFS.
///
/// # Arguments
/// * `node_count` - Total number of nodes in the graph
/// * `adjacency_fn` - Function that returns neighbors for a given node ID
/// * `node_label_fn` - Function that converts node index to human-readable label
///
/// # Returns
/// Vector of cycles, where each cycle is a vector of node labels.
///
/// # Complexity
/// O(V + E) time, O(V) space
pub fn find_cycles<F, L>(node_count: usize, adjacency_fn: F, node_label_fn: L) -> Vec<Vec<String>>
where
    F: Fn(usize) -> Vec<usize>,
    L: Fn(usize) -> String,
{
    if node_count == 0 {
        return Vec::new();
    }

    let mut color: HashMap<usize, u8> = HashMap::with_capacity(node_count);
    for i in 0..node_count {
        color.insert(i, 0);
    }

    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut path_stack: Vec<usize> = Vec::new();

    fn dfs_find<F, L>(
        node: usize,
        adj_fn: &F,
        label_fn: &L,
        color: &mut HashMap<usize, u8>,
        path_stack: &mut Vec<usize>,
        cycles: &mut Vec<Vec<String>>,
    ) where
        F: Fn(usize) -> Vec<usize>,
        L: Fn(usize) -> String,
    {
        color.insert(node, 1);
        path_stack.push(node);

        let neighbors = adj_fn(node);
        for next in neighbors {
            match color.get(&next).copied().unwrap_or(0) {
                1 => {
                    let pos = path_stack.iter().position(|&n| n == next);
                    if let Some(start) = pos {
                        let cycle: Vec<String> =
                            path_stack[start..].iter().map(|&n| label_fn(n)).collect();
                        cycles.push(cycle);
                    }
                }
                0 => {
                    dfs_find(next, adj_fn, label_fn, color, path_stack, cycles);
                }
                _ => {}
            }
        }

        path_stack.pop();
        color.insert(node, 2);
    }

    for node in 0..node_count {
        if color.get(&node).copied().unwrap_or(0) == 0 {
            dfs_find(
                node,
                &adjacency_fn,
                &node_label_fn,
                &mut color,
                &mut path_stack,
                &mut cycles,
            );
        }
    }

    cycles
}

/// Compute transitive dependencies using BFS.
///
/// # Arguments
/// * `start_node` - Starting node index
/// * `max_depth` - Maximum depth to traverse (0 or negative = unlimited)
/// * `node_count` - Total number of nodes
/// * `adjacency_fn` - Function that returns neighbors for a given node ID
///
/// # Returns
/// Vector of reachable node indices (excluding start_node)
pub fn transitive_dependencies<F>(
    start_node: usize,
    max_depth: i32,
    node_count: usize,
    adjacency_fn: F,
) -> Vec<usize>
where
    F: Fn(usize) -> Vec<usize>,
{
    if node_count == 0 {
        return Vec::new();
    }

    let mut visited = vec![false; node_count];
    let mut queue: VecDeque<(usize, i32)> = VecDeque::new();
    let mut result: Vec<usize> = Vec::new();

    visited[start_node] = true;
    queue.push_back((start_node, 0));

    while let Some((current, depth)) = queue.pop_front() {
        if max_depth > 0 && depth >= max_depth {
            continue;
        }

        let neighbors = adjacency_fn(current);
        for next in neighbors {
            if !visited[next] {
                visited[next] = true;
                result.push(next);
                queue.push_back((next, depth + 1));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_cycle_empty_graph() {
        assert!(!has_cycle(0, |_| vec![]));
    }

    #[test]
    fn has_cycle_single_node() {
        assert!(!has_cycle(1, |_| vec![]));
    }

    #[test]
    fn has_cycle_two_nodes_no_cycle() {
        let adj = |i: usize| match i {
            0 => vec![1],
            _ => vec![],
        };
        assert!(!has_cycle(2, adj));
    }

    #[test]
    fn has_cycle_two_nodes_with_cycle() {
        let adj = |i: usize| match i {
            0 => vec![1],
            1 => vec![0],
            _ => vec![],
        };
        assert!(has_cycle(2, adj));
    }

    #[test]
    fn has_cycle_three_node_cycle() {
        let adj = |i: usize| match i {
            0 => vec![1],
            1 => vec![2],
            2 => vec![0],
            _ => vec![],
        };
        assert!(has_cycle(3, adj));
    }

    #[test]
    fn find_cycles_empty_graph() {
        let cycles = find_cycles(0, |_| vec![], |i| i.to_string());
        assert!(cycles.is_empty());
    }

    #[test]
    fn find_cycles_simple_cycle() {
        let adj = |i: usize| match i {
            0 => vec![1],
            1 => vec![0],
            _ => vec![],
        };
        let cycles = find_cycles(2, adj, |i| i.to_string());
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec!["0", "1"]);
    }

    #[test]
    fn transitive_dependencies_empty() {
        let result = transitive_dependencies(0, 1, 0, |_| vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn transitive_dependencies_depth_1() {
        let adj = |i: usize| match i {
            0 => vec![1, 2],
            1 => vec![3],
            _ => vec![],
        };
        let result = transitive_dependencies(0, 1, 4, adj);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn transitive_dependencies_depth_2() {
        let adj = |i: usize| match i {
            0 => vec![1, 2],
            1 => vec![3],
            _ => vec![],
        };
        let result = transitive_dependencies(0, 2, 4, adj);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn transitive_dependencies_unlimited() {
        let adj = |i: usize| match i {
            0 => vec![1],
            1 => vec![2],
            2 => vec![3],
            _ => vec![],
        };
        let result = transitive_dependencies(0, 0, 4, adj);
        assert_eq!(result, vec![1, 2, 3]);
    }
}
