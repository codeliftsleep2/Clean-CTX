//! Trusted-local path and finite deny-signature enforcement. Not a secret-free proof.
use super::{Error, Result, require};
use std::path::{Component, Path, PathBuf};

pub(super) fn relative(path: &str) -> Result<()> {
    require(
        !path.is_empty() && !path.contains(['\\', ':', '\0']) && !path.starts_with('/'),
        "unsafe_path",
    )?;
    for part in path.split('/') {
        require(
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.to_uppercase() != ".GIT"
                && !part.ends_with(['.', ' ']),
            "unsafe_path",
        )?;
        let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        require(
            !["CON", "PRN", "AUX", "NUL"].contains(&base.as_str())
                && !(base.len() == 4
                    && (base.starts_with("COM") || base.starts_with("LPT"))
                    && matches!(base.as_bytes()[3], b'1'..=b'9')),
            "unsafe_path",
        )?;
    }
    Ok(())
}

pub(super) fn no_links(path: &Path, allow_missing_leaf: bool) -> Result<()> {
    let mut current = PathBuf::new();
    let count = path.components().count();
    for (i, part) in path.components().enumerate() {
        require(
            !matches!(part, Component::ParentDir | Component::CurDir),
            "unsafe_path",
        )?;
        current.push(part);
        // A Windows drive/UNC prefix alone is not yet an absolute directory;
        // especially verbatim `\\?\C:` cannot be queried before its root separator.
        if matches!(part, Component::Prefix(_)) {
            continue;
        }
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                require(!meta.file_type().is_symlink(), "unsafe_path")?;
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    require(meta.file_attributes() & 0x400 == 0, "unsafe_path")?;
                }
                if i + 1 < count {
                    require(meta.is_dir(), "unsafe_path")?;
                }
            }
            Err(e)
                if allow_missing_leaf
                    && i + 1 == count
                    && e.kind() == std::io::ErrorKind::NotFound =>
            {}
            Err(_) => return Err(Error("source_inaccessible")),
        }
    }
    Ok(())
}

pub(super) fn source_path(root: &Path, path: &str) -> Result<PathBuf> {
    relative(path)?;
    let joined = root.join(path);
    no_links(&joined, false)?;
    let canonical = joined
        .canonicalize()
        .map_err(|_| Error("source_inaccessible"))?;
    require(
        canonical.starts_with(root) && canonical.is_file(),
        "unsafe_path",
    )?;
    let relative = canonical
        .strip_prefix(root)
        .map_err(|_| Error("unsafe_path"))?;
    require(
        !relative
            .components()
            .any(|c| c.as_os_str().to_string_lossy().to_uppercase() == ".GIT"),
        "unsafe_path",
    )?;
    Ok(canonical)
}

/// Windows `Permissions` exposes only the readonly bit. Include owner/group/DACL
/// so a readable-but-expanded ACL cannot silently pass profile revalidation.
#[cfg(windows)]
pub(super) fn permissions(path: &Path) -> Result<Vec<u8>> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        #[link_name = "GetFileSecurityW"]
        fn get_file_security(
            name: *const u16,
            information: u32,
            descriptor: *mut u8,
            length: u32,
            needed: *mut u32,
        ) -> i32;
    }
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut needed = 0;
    // SAFETY: name is NUL-terminated; the null descriptor/zero length requests
    // only the required size, written into a live DWORD. No SACL privilege needed.
    unsafe {
        get_file_security(name.as_ptr(), 7, std::ptr::null_mut(), 0, &mut needed);
    }
    require(needed > 0, "profile_permissions")?;
    let mut descriptor = vec![0u8; needed as usize];
    // SAFETY: descriptor owns `needed` writable bytes for this synchronous call.
    let success = unsafe {
        get_file_security(
            name.as_ptr(),
            7,
            descriptor.as_mut_ptr(),
            needed,
            &mut needed,
        )
    };
    require(
        success != 0 && needed as usize <= descriptor.len(),
        "profile_permissions",
    )?;
    descriptor.truncate(needed as usize);
    Ok(descriptor)
}

#[cfg(unix)]
pub(super) fn permissions(path: &Path) -> Result<Vec<u8>> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path).map_err(|_| Error("profile_permissions"))?;
    Ok([metadata.uid(), metadata.gid(), metadata.mode()]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect())
}

#[cfg(not(any(windows, unix)))]
pub(super) fn permissions(_: &Path) -> Result<Vec<u8>> {
    Err(Error("profile_permissions"))
}

pub(super) fn json_nonsecret(value: &serde_json::Value) -> Result<()> {
    match value {
        serde_json::Value::String(s) => nonsecret(s),
        serde_json::Value::Array(items) => {
            for item in items {
                json_nonsecret(item)?;
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                nonsecret(key)?;
                json_nonsecret(value)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(unix)]
pub(super) fn physical_identity(path: &Path) -> Result<Vec<u8>> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path).map_err(|_| Error("source_inaccessible"))?;
    Ok([metadata.dev(), metadata.ino()]
        .into_iter()
        .flat_map(u64::to_le_bytes)
        .collect())
}

#[cfg(windows)]
pub(super) fn physical_identity(path: &Path) -> Result<Vec<u8>> {
    use std::os::windows::io::AsRawHandle;
    #[repr(C)]
    #[derive(Default)]
    struct FileInformation {
        volume: u64,
        file_id: [u8; 16],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetFileInformationByHandleEx"]
        fn get_file_information(
            handle: *mut std::ffi::c_void,
            class: i32,
            information: *mut FileInformation,
            size: u32,
        ) -> i32;
    }
    let file = std::fs::File::open(path).map_err(|_| Error("source_inaccessible"))?;
    let mut information = FileInformation::default();
    // SAFETY: file owns the live handle; the repr(C) structure has the documented
    // FILE_ID_INFO layout and remains writable during this call. FileIdInfo=18
    // uses all 128 file-ID bits (including ReFS), rather than a truncated index.
    require(
        unsafe {
            get_file_information(
                file.as_raw_handle(),
                18,
                &mut information,
                std::mem::size_of::<FileInformation>() as u32,
            )
        } != 0,
        "source_inaccessible",
    )?;
    Ok([
        information.volume.to_le_bytes().as_slice(),
        information.file_id.as_slice(),
    ]
    .concat())
}

#[cfg(not(any(windows, unix)))]
pub(super) fn physical_identity(_: &Path) -> Result<Vec<u8>> {
    Err(Error("source_inaccessible"))
}

// Equivalent byte predicates for the four finite Rust-regex deny expressions in
// design §10.2; no regular-expression dependency or model redaction is involved.
pub(super) fn nonsecret(text: &str) -> Result<()> {
    let private = ["", "RSA ", "EC ", "OPENSSH "]
        .iter()
        .any(|kind| text.contains(&format!("-----BEGIN {kind}PRIVATE KEY-----")));
    require(!private, "material_denied")?;
    let bytes = text.as_bytes();
    for i in 0..bytes.len() {
        let tail = &bytes[i..];
        let token = |prefix: &[u8], minimum: usize, underscore: bool, dash: bool| {
            tail.starts_with(prefix)
                && tail[prefix.len()..]
                    .iter()
                    .take_while(|b| {
                        b.is_ascii_alphanumeric()
                            || (underscore && **b == b'_')
                            || (dash && **b == b'-')
                    })
                    .count()
                    >= minimum
        };
        require(
            !(token(b"github_pat_", 20, true, false)
                || token(b"sk-", 20, true, true)
                || [b"ghp_", b"gho_", b"ghu_", b"ghs_", b"ghr_"]
                    .iter()
                    .any(|p| token(p.as_slice(), 20, false, false))
                || (tail.starts_with(b"AKIA")
                    && tail[4..]
                        .iter()
                        .take_while(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
                        .count()
                        >= 16)),
            "material_denied",
        )?;
    }
    // Rust-regex Unicode simple case folding adds long-s and Kelvin sign to
    // ASCII s/k equivalence classes. This is a detection view, never stored text.
    let lower: String = text
        .chars()
        .map(|ch| match ch {
            'ſ' => 's',
            'K' => 'k',
            ch => ch.to_ascii_lowercase(),
        })
        .collect();
    for (start, _) in lower.char_indices() {
        let rest = &lower[start..];
        if let Some(after) = rest.strip_prefix("authorization") {
            if let Some(after) = after
                .trim_start_matches(char::is_whitespace)
                .strip_prefix(':')
            {
                if let Some(after) = after
                    .trim_start_matches(char::is_whitespace)
                    .strip_prefix("bearer")
                {
                    require(
                        !after.starts_with(char::is_whitespace)
                            || after.trim_start_matches(char::is_whitespace).is_empty(),
                        "material_denied",
                    )?;
                }
            }
        }
        for key in [
            "api_key",
            "api-key",
            "apikey",
            "client_secret",
            "client-secret",
            "clientsecret",
            "password",
            "access_token",
            "access-token",
            "accesstoken",
        ] {
            if let Some(after) = rest.strip_prefix(key) {
                let after = after.trim_start_matches(char::is_whitespace);
                if after.starts_with([':', '=']) {
                    let raw = after[1..].trim_start_matches(char::is_whitespace);
                    let value = raw
                        .strip_prefix('\'')
                        .or_else(|| raw.strip_prefix('"'))
                        .unwrap_or(raw);
                    let count = value
                        .chars()
                        .take_while(|c| !c.is_whitespace() && *c != '\'' && *c != '"')
                        .count();
                    require(count < 8, "material_denied")?;
                }
            }
        }
    }
    Ok(())
}
