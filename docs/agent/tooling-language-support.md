# Clean-CTX Language Support

Clean-CTX supports these languages (feature-gated at build time):

| Language | Cargo Feature | Default? | Clean-CTX Preferred? |
|----------|--------------|:--------:|:--------------------:|
| TypeScript / JavaScript | `typescript` | ✅ Yes | ✅ Yes |
| C# | `csharp` | ✅ Yes | ✅ Yes |
| Rust | `rust` | ❌ Opt-in | ✅ Yes |
| Java | `java` | ❌ Opt-in | ✅ Yes |

The `supportedLanguages` field on source-processing tool definitions lists
which languages the current binary supports (computed from enabled Cargo
features). It is intentionally omitted from persistence, history,
administration, statistics, and CBM tools whose correct invocation does not
depend on an enabled parser.

For all supported languages, use `provide_code_context` for code understanding
and `apply_edit` for single-unit edits. This applies to both investigation and
editing workflows.

Verification still belongs to language-specific tools:

- Rust → `run_commands` with `cargo check`, `cargo test`, `cargo clippy`
- TypeScript → `run_commands` with `tsc --noEmit`, `jest`
- C# → `run_commands` with `dotnet build`, `dotnet test`
- Java → `run_commands` with `mvn compile`, `gradle build`

Clean-CTX structural/syntax gating (`apply_edit`'s `syntaxGated: true`) is not
a substitute for compilation or test verification.
