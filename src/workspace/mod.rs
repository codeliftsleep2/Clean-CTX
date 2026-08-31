// src/workspace/mod.rs
//
// WorkspaceIndex — framework-agnostic cross-file semantic index.
//
// The WorkspaceIndex aggregates SemanticEdge objects from all files in a
// workspace, deduplicates them by identity, and provides entity/edge
// lookup. It is the foundation for cross-file semantic queries without
// CBM.
//
// Phase 4a establishes the core: entity/edge storage, deduplication, file
// provenance, forward/reverse indexes, and deterministic ordering.
// Phase 4b adds query APIs; Phase 4c adds advanced graph algorithms.
//
// This module is framework-agnostic: it works with any SemanticEdge
// regardless of domain (angular, dotnet, spring, ngrx).

pub mod index;
