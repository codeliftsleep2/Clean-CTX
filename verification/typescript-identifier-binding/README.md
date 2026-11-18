# TypeScript identifier-binding verification

This tracked fixture corpus verifies the private #140e-Prereq2a same-file
binding boundary. The Rust harness reads real `.ts` sources, obtains reference
spans from the existing TypeScript invocation-evidence pipeline, and compares
binding results with the independent marker-based oracle in `oracle.tsv`.

Markers label controlled source positions but are comments only: they do not
participate in parsing or binding. Positive rows assert exact reference and
declaration spans, source identity, source hash, and declaration kind. Refusal
rows preserve the intentionally unsupported boundary. The removal fixtures
prove that the winner moves outward when a supported inner declaration is
absent. Separate files prove that same-named declarations never cross source
identity.

The fixture harness is operator/test evidence, not a public binding API and not
an NgRx composition. It does not use `first_body_token`, WorkspaceIndex,
persistence, MCP, module resolution, or framework classification.
