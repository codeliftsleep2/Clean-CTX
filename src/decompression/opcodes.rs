// src/decompression/opcodes.rs
//
// Backward-compatible re-export shim. The 32-entry primitive opcode table
// was the single source of truth for the `Decompressor`; in Phase 2 it
// moves to `crate::compression::opcodes` so the dictionary and the
// decompressor share one definition.
//
// This file re-exports the shared items under the old names so that any
// existing `use crate::decompression::opcodes::...` imports continue to
// compile. The shim will be deleted in a later phase once internal
// callers are migrated.

pub(crate) use crate::compression::opcodes::builtin_opcode_map;
