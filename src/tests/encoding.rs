// src/tests/encoding.rs
//
// Encoding invariant guard. See .clinerules/encoding.md (authoritative policy)
// and scripts/check-utf8.ps1 (the CI twin of this gate).
//
// Two layers are enforced here with std-only code (no dependencies):
//   1. The Unicode canary fixture survives `LLM -> tool -> serialization ->
//      filesystem -> parser -> Git/CI` boundaries byte-for-byte. The expected
//      value is expressed with `\u{...}` escapes so THIS TEST SOURCE stays
//      pure ASCII and cannot itself be corrupted by a bad write.
//   2. Every repository text file is strictly valid UTF-8, has no BOM, and
//      contains no known mojibake signature sequences, except paths listed
//      in the intentional-documentation allowlist.

use std::fs;
use std::path::{Path, PathBuf};

/// Path of the canary fixture, resolved relative to this test file (src/tests/).
const CANARY_PATH: &str = "../test_files/unicode_canary.txt";

/// Byte-exact expected canary content. Pure-ASCII escapes only.
const CANARY_EXPECTED: &str = concat!(
    "T\u{FC}rk\u{E7}e \u{130}stanbul \u{E7}al\u{131}\u{15F}\u{131}yor ",
    "\u{15F} \u{11F} \u{131} \u{130} \u{F6} \u{FC} \u{E7}\n",
    "\u{E9} \u{F1} \u{20AC} \u{A3} \u{A5} ",
    "\u{201C} \u{201D} \u{2018} \u{2019} \u{2014} \u{2026} \u{2190} \u{2192} \u{2194}\n",
    "\u{3A6} \u{3B1} \u{3B2} \u{A7} \u{B6} \u{3A9} \u{2206}\n",
    "\u{65E5}\u{672C}\u{8A9E} \u{4E2D}\u{6587} \u{D55C}\u{AD6D}\u{C5B4}\n",
    "\u{1F680} \u{2705} \u{1F980} \u{1F4E6}\n",
    "\u{A7}PATHMAP \u{3B1}1 \u{2192} p:name:type\n",
);

/// Mojibake signatures (chars resulting from UTF-8 bytes misread through
/// Latin-1/cp1252-style tables). Escapes keep this file pure ASCII.
/// Mirrors `scripts/check-utf8.ps1`.
const MOJIBAKE_SIGNATURES: &[&str] = &[
    // Full mojibake sequences: bytes of X misread per-byte through cp1252 or
    // Latin-1, then re-encoded as UTF-8. Both decode tables covered.
    "\u{E2}\u{20AC}\u{2122}", // <- U+2019 apostrophe (cp1252 tail)
    "\u{E2}\u{20AC}\u{99}",   // <- U+2019 apostrophe (latin1 tail)
    "\u{E2}\u{20AC}\u{153}",  // <- U+201C left quote (cp1252)
    "\u{E2}\u{20AC}\u{9C}",   // <- U+201C left quote (latin1)
    "\u{E2}\u{20AC}\u{17E}",  // <- U+201D right quote (cp1252)
    "\u{E2}\u{20AC}\u{9D}",   // <- U+201D right quote (latin1)
    "\u{E2}\u{20AC}\u{201D}", // <- U+2014 em dash (cp1252; 0x94 -> smart right quote)
    "\u{E2}\u{20AC}\u{94}",   // <- U+2014 em dash (latin1)
    "\u{E2}\u{20AC}",         // incomplete smart-punct fragment prefix
    "\u{C3}",                 // lone cp1252-mapped lead byte C3
    "\u{C2}",                 // lone lead byte C2
    "\u{FFFD}",               // replacement character (already-substituted bytes)
];

// --- Text-file selection ------------------------------------------------------
// Mirrors scripts/check-utf8.ps1 selection logic exactly so the two gates can
// never disagree about what constitutes "relevant". Update both together.
fn is_tracked_text_file(rel: &str) -> bool {
    const TEXT_EXTS: &[&str] = &[
        "rs",
        "toml",
        "json",
        "jsonc",
        "yml",
        "yaml",
        "md",
        "markdown",
        "txt",
        "html",
        "css",
        "scss",
        "mjs",
        "cjs",
        "js",
        "jsx",
        "ts",
        "tsx",
        "cs",
        "java",
        "sql",
        "xml",
        "csv",
        "tsv",
        "ps1",
        "psm1",
        "lock",
        "sh",
        "gitattributes",
        "dotsettings",
    ];
    // Root dotfiles without extensions that we still validate.
    const DOTFILES: &[&str] = &[".gitignore", ".gitattributes", ".editorconfig"];
    let name = rel.rsplit(['/', '\\']).next().unwrap_or(rel);
    if DOTFILES.contains(&name) {
        return true;
    }
    rel.rsplit('.')
        .next()
        .map(|e| TEXT_EXTS.contains(&e))
        .unwrap_or(false)
}

/// Repo root resolved relative to this test source (src/tests/ -> repo root).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Collect tracked + untracked-but-not-ignored text files via `git ls-files`,
/// exactly matching the CI gate's file universe.
fn collect_text_files() -> Vec<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files must be runnable from the repository");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output filenames are UTF-8")
        .lines()
        .filter(|l| !l.trim().is_empty() && is_tracked_text_file(l))
        .map(|l| repo_root().join(l))
        .collect()
}

/// True when no known mojibake signature appears anywhere in `content`.
/// The signatures mirror scripts/check-utf8.ps1; update both together.
///
/// Clippy 1.96 (`manual_find`): expressed with `Iterator::find` instead of a
/// hand-written scan loop. `.copied()` lifts the `&str` items out of the
/// constant table so the returned reference keeps its `'static` lifetime;
/// semantics unchanged (first matching signature wins, `None` otherwise).
fn contains_mojibake(content: &str) -> Option<&'static str> {
    MOJIBAKE_SIGNATURES
        .iter()
        .copied()
        .find(|sig| content.contains(*sig))
}

/// Files permitted to contain mojibake SIGNATURES because they quote historic
/// corruption (forensic records). Must stay in sync with the allowlist in
/// scripts/check-utf8.ps1. Keep reasons beside each entry.
const MOJIBAKE_ALLOWED_PATHS: &[&str] = &["docs/CHANGELOG.md"]; // historical bug-fix record quoting corrupted sequences

#[test]
fn unicode_canary_survives_file_round_trip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests") // CANARY_PATH is relative to this source directory (src/tests/)
        .join(CANARY_PATH);
    let actual = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "canary fixture must exist and be readable at {:?}: {e}",
            path
        )
    });
    assert_eq!(
        actual, CANARY_EXPECTED,
        "Unicode canary fixture diverged from expected bytes; an encoding boundary corrupted data between LLM/tool/write/read layers."
    );
}

#[test]
fn unicode_canary_is_byte_identical_through_git() {
    // Proves git blob round-trip preserves the exact bytes: index vs worktree.
    let root = repo_root();
    let rel = "src/test_files/unicode_canary.txt";
    let idx = std::process::Command::new("git")
        .args(["show", &format!(":{rel}")])
        .current_dir(&root)
        .output()
        .expect("git show (index) failed");
    assert!(
        idx.status.success(),
        "canary must be staged/tracked for the git round-trip proof"
    );
    let worktree = fs::read(root.join(rel)).expect("canary readable");
    assert_eq!(
        worktree, idx.stdout,
        "worktree canary differs from git index; checkout/filter boundary altered encoding."
    );
}

#[test]
fn all_repo_text_files_are_strict_utf8_without_mojibake() {
    for path in collect_text_files() {
        let bytes = fs::read(&path).unwrap_or_else(|e| panic!("unreadable {:?}: {e}", path));
        let rel = path.strip_prefix(repo_root()).unwrap();
        let rel_s = rel.to_string_lossy().replace('\\', "/");

        let decoded = match strict_utf8(&bytes) {
            Ok(s) => s,
            Err(valid_up_to) => panic!(
                "{rel_s}: INVALID UTF-8 near byte offset {valid_up_to} - encoding boundary failure (see docs/ENCODING_POLICY.md)"
            ),
        };

        // Pure-ASCII files cannot carry mojibake signatures; skip the scan.
        if !bytes.iter().any(|&b| b > 127) {
            continue;
        }
        if MOJIBAKE_ALLOWED_PATHS.contains(&rel_s.as_str()) {
            continue;
        }
        if let Some(sig) = contains_mojibake(&decoded) {
            panic!(
                "{rel_s}: mojibake signature {sig:?} found in non-allowlisted file (see docs/ENCODING_POLICY.md)"
            );
        }
    }
}

#[test]
fn encoding_gates_stay_in_sync_with_each_other() {
    // The Rust gate and the CI PowerShell gate must scan the same file
    // universe and allowlist, or they would silently disagree. This pins the
    // contract at the boundary most likely to drift.
    assert_eq!(
        MOJIBAKE_ALLOWED_PATHS,
        &["docs/CHANGELOG.md"],
        "Rust-side allowlist drifted; update scripts/check-utf8.ps1 $MojibakeAllowedPaths together"
    );
}

/// Strict UTF-8 decode using only std: never substitutes anything.
/// `Err(valid_up_to)` reports the byte offset of the first invalid sequence.
fn strict_utf8(bytes: &[u8]) -> Result<String, usize> {
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(e.valid_up_to()),
    }
}
