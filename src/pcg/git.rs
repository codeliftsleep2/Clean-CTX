//! Local-only Git object acquisition. No shell, hooks, replacement objects or lazy fetch.
use super::{
    Error, Result, identity,
    model::Manifest,
    profile::{Trusted, Work},
    require, security,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    process::{Command, Stdio},
};

pub(super) fn command(trusted: &Trusted, args: &[&str], limit: u64) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    command
        .arg("--no-replace-objects")
        .arg("--no-optional-locks")
        .arg("--literal-pathspecs")
        .arg("-C")
        .arg(&trusted.root)
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_COUNT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(|_| Error("git_unavailable"))?;
    let mut output = Vec::new();
    let read = child
        .stdout
        .take()
        .ok_or(Error("git_unavailable"))?
        .take(limit)
        .read_to_end(&mut output);
    if read.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(Error("git_output_limit"));
    }
    require(
        child
            .wait()
            .map_err(|_| Error("git_unavailable"))?
            .success(),
        "git_object_invalid",
    )?;
    Ok(output)
}

pub(super) fn sources(
    trusted: &Trusted,
    manifest: &Manifest,
    work: &mut Work,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let root = command(
        trusted,
        &["rev-parse", "--show-toplevel"],
        trusted.root.as_os_str().len() as u64 + 32,
    )?;
    let root = std::str::from_utf8(&root)
        .map_err(|_| Error("git_root"))?
        .trim();
    require(
        std::path::Path::new(root)
            .canonicalize()
            .map_err(|_| Error("git_root"))?
            == trusted.root,
        "git_root",
    )?;
    let format = command(trusted, &["rev-parse", "--show-object-format"], 16)?;
    let oid_len = match format.as_slice() {
        b"sha1\n" => 40,
        b"sha256\n" => 64,
        _ => return Err(Error("git_object_format")),
    };
    let mut commits = BTreeSet::new();
    let mut blobs = BTreeMap::new();
    for source in &manifest.sources {
        identity::hex(&source.commit, oid_len)?;
        identity::hex(&source.blob, oid_len)?;
        security::relative(&source.path)?;
        if commits.insert(&source.commit) {
            require(
                command(trusted, &["cat-file", "-t", &source.commit], 16)? == b"commit\n",
                "git_commit_invalid",
            )?;
        }
        let entry = command(
            trusted,
            &["ls-tree", "-z", &source.commit, "--", &source.path],
            (source.path.len() as u64).saturating_add(256),
        )?;
        let tab = entry
            .iter()
            .position(|b| *b == b'\t')
            .ok_or(Error("git_membership"))?;
        let expected = format!(" blob {}", source.blob);
        require(
            (entry.starts_with(b"100644") || entry.starts_with(b"100755"))
                && entry[..tab].ends_with(expected.as_bytes())
                && entry[tab + 1..] == [source.path.as_bytes(), &[0]].concat(),
            "git_membership",
        )?;
        let remaining = work.ceiling.saturating_sub(work.bytes);
        let size = command(trusted, &["cat-file", "-s", &source.blob], 32)?;
        let size = std::str::from_utf8(&size)
            .map_err(|_| Error("git_object_invalid"))?
            .trim()
            .parse::<u64>()
            .map_err(|_| Error("git_object_invalid"))?;
        require(size <= remaining, "work_limit")?;
        // Git's local cat-file producer emits the immutable object's declared
        // size. Consume only that size; no source-byte lookahead crosses budget.
        let bytes = command(trusted, &["cat-file", "blob", &source.blob], size)?;
        require(bytes.len() as u64 == size, "git_object_invalid")?;
        work.debit(bytes.len() as u64)?;
        work.git_bytes += bytes.len() as u64;
        require(identity::sha(&bytes) == source.sha256, "source_hash")?;
        require(
            !bytes.starts_with(&[0xef, 0xbb, 0xbf]) && std::str::from_utf8(&bytes).is_ok(),
            "source_utf8",
        )?;
        // Profile, manifest and derived database may never be enrolled as evidence.
        let path = trusted.root.join(&source.path);
        for excluded in [
            trusted.path.as_path(),
            std::path::Path::new(&trusted.profile.manifest_path),
            std::path::Path::new(&trusted.profile.database_path),
        ] {
            let excluded = excluded
                .canonicalize()
                .unwrap_or_else(|_| excluded.to_owned());
            let candidate = path.canonicalize().unwrap_or_else(|_| path.clone());
            require(candidate != excluded, "profile_isolation")?;
        }
        require(
            source.check != "historical" || trusted.profile.history_allowed,
            "history_denied",
        )?;
        blobs.insert(source.id.clone(), bytes);
    }
    Ok(blobs)
}

/// A boundary observation, not an atomic snapshot or ABA detector.
pub(super) fn current(
    trusted: &Trusted,
    manifest: &Manifest,
    work: &mut Work,
    first: bool,
) -> Result<()> {
    let mut paths = BTreeMap::new();
    for source in manifest.sources.iter().filter(|s| s.check == "current") {
        let path = security::source_path(&trusted.root, &source.path)?;
        let key = security::physical_identity(&path)?;
        if let Some((_, hash)) = paths.insert(key, (path, &source.sha256)) {
            require(hash == &source.sha256, "current_alias_conflict")?;
        }
    }
    if first {
        work.current_sources = paths.len() as u64;
    } else {
        work.current_sources_completion = paths.len() as u64;
    }
    for (_, (path, hash)) in paths {
        let bytes = work.read(&path)?;
        work.current_bytes += bytes.len() as u64;
        require(identity::sha(&bytes) == *hash, "source_stale")?;
    }
    Ok(())
}
