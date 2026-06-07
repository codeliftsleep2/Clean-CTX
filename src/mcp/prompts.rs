// src/mcp/prompts.rs
//
// Prompt content for the MCP server.

/// The cleanctx-notation system prompt.
pub(crate) const SYSTEM_PROMPT: &str = concat!(
    "# Clean-CTX Notation Guide\n\n",
    "You are working with Clean-CTX compressed code notation. ",
    "This is an AST-based compression format that strips implementation details while preserving structural signatures.\n\n",
    "## Opcode Reference\n\n",
    "### Built-in Primitives (always available)\n",
    "| Opcode | Token | Opcode | Token | Opcode | Token |\n",
    "|--------|-------|--------|-------|--------|-------|\n",
    "| $c | class | $s | string | $b | boolean |\n",
    "| $n | number | $v | void | $a | async |\n",
    "| $e | export | $r | return | $t | throw |\n",
    "| $T | true | $F | false | $P | Promise |\n",
    "| $ctor | constructor | $fn | function | $E | Error |\n",
    "| $nw | new | $i | if | $fr | for |\n",
    "| $w | while | $h | this | $k | const |\n",
    "| $l | let | $pu | public | $pv | private |\n",
    "| $st | static | $x | extends | $m | implements |\n",
    "| $if | interface | $ty | type | $nl | null |\n",
    "| $ud | undefined | $fm | from | $im | import |\n\n",
    "### Custom Opcodes\n",
    "Custom opcodes ($1, $2, ...) are auto-assigned to tokens appearing 2+ times in the session. ",
    "Check the §SYM footer for custom opcode definitions.\n\n",
    "### Path Aliases\n",
    "File paths are compressed to α1, α2, β1, etc. Check §MAP footer for path mappings.\n\n",
    "## Behavior Markers\n",
    "| Marker | Meaning |\n",
    "|--------|---------|\n",
    "| ⊕guard | Conditional branch (if statement) |\n",
    "| ⊕loop | Iteration (for/while) |\n",
    "| ⊕⇒ | Return value follows |\n",
    "| ⊕! | Throws error |\n",
    "| ⊕export | Module export |\n\n",
    "## Diff Markers (from diff_code_context)\n",
    "| Marker | Meaning |\n",
    "|--------|---------|\n",
    "| + | Added (new class, method, field, or import) |\n",
    "| - | Removed |\n",
    "| ~ | Modified (signature or markers changed) |\n",
    "| = | Unchanged (included for scope context) |\n\n",
    "## Rules for Using Compressed Notation\n",
    "1. When reading compressed context, interpret opcodes using the tables above\n",
    "2. When writing code in compressed form, use the opcodes and markers\n",
    "3. NEVER output raw opcode tables or §MAP/§SYM footers — those are internal metadata\n",
    "4. When asked to expand, use the decompress_code_context tool\n",
    "5. When asked for changes between versions, use the diff_code_context tool — it returns only the deltas\n",
    "6. Preserve the semantic meaning — compressed ≠ less accurate\n",
    "7. Use the same fidelity level as the compressed context you received\n\n",
    "## Example\n",
    "Compressed: `$c UserService;$ctor();$a process(payload: $s[]): $P<$b>`\n",
    "Interpreted: `class UserService; constructor(); async process(payload: string[]): Promise<boolean>`\n",
    "Write back as: `$c UserService { $ctor() $a process(payload: $s[]): $P<$b> }`\n",
    "Diff:        `~ class UserService\\n  ~ method process: process(payload: $s[]): $P<$b>\\n    was: process(payload: $s[]): $P<$s>`\n",
);

/// Return the list of available prompt definitions (for `prompts/list`).
pub(crate) fn prompt_list() -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "name": "cleanctx-notation",
        "description": "System instructions for reading and writing Clean-CTX compressed notation",
        "arguments": []
    })]
}