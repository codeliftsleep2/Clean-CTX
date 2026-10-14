# NgRx Store receiver qualification verification

The tracked Rust harnesses
`src/tests/angular_meta/ngrx_store_receiver_qualification.rs` and
`src/tests/mcp/ngrx_store_receiver_qualification.rs` exercise real TypeScript
compilation and the registered MCP boundary. Independent expectations cover
arbitrary receiver names, foreign and lookalike Stores, wrong symbols, member
and class collisions, unsupported imports and `this` contexts, and current-
source mutation.

Only direct `this.<property>.select(...)` operations with exact Prereq3a member
evidence, exact Prereq3b `Store` provenance from `@ngrx/store`, and independent
Prereq2 Selector authority produce durable `Selects` semantics. Marker-level
candidate occurrences remain separate. Service ownership is not established.
