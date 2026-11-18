# TypeScript type-reference to named-import verification

The tracked Rust harness at
`src/tests/ir/typescript_type_import_binding.rs` uses real TypeScript parsing
and independent source-offset expectations. It covers direct and generic type
heads, multiple imports and symbols, exact foreign/barrel module provenance,
type-space collisions, ambiguity, cross-file identity, and refusal of alias,
namespace, default, type-only, inline-type, and missing imports.

One integration case begins with the exact type-reference span returned by the
Prereq3a `this.<property>` binder. The resulting evidence remains private,
same-file, source-hashed, and framework-blind. It performs no module traversal
or semantic classification.
