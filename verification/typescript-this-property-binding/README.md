# TypeScript `this.<property>` binding verification

The tracked Rust harness at
`src/tests/ir/typescript_this_property_binding.rs` drives real TypeScript
sources through the parser and the private member binder. Its independent
source-offset assertions cover parameter-property modifiers, arbitrary names,
same-name classes, lexical shadows, similar names, missing and ambiguous
members, unsafe `this` contexts, unsupported receivers, and an exact receiver
span produced by `InvocationOccurrence`.

This evidence is compilation-local and framework-blind. It does not resolve
types to imports, classify NgRx Store receivers, change public MCP behavior,
or persist member bindings.
