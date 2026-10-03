#[cfg(any(windows, unix))]
use std::collections::HashMap;

#[cfg(any(windows, unix))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PhysicalFileIdentity {
    filesystem_id: u64,
    file_id: u64,
}

#[cfg(any(windows, unix))]
#[derive(Debug)]
struct PhysicalOwner {
    semantic_path: String,
    durable_path: Option<String>,
}

/// Session-local bridge from physical file identity to the stable canonical
/// path that owns Clean-CTX semantic and durable state. Paths remain the public,
/// provenance, and durable representation; the file identity is lookup-only.
#[derive(Debug, Default)]
pub(super) struct PhysicalIdentityRegistry {
    #[cfg(any(windows, unix))]
    owners: HashMap<PhysicalFileIdentity, PhysicalOwner>,
}

impl PhysicalIdentityRegistry {
    pub(super) fn resolve_semantic_owner(&mut self, path: &str) -> String {
        let canonical = crate::dictionary::path::canonical_identity_key(path);
        #[cfg(any(windows, unix))]
        if let Some(identity) = physical_file_identity(&canonical) {
            return self
                .owners
                .entry(identity)
                .or_insert_with(|| PhysicalOwner {
                    semantic_path: canonical.clone(),
                    durable_path: None,
                })
                .semantic_path
                .clone();
        }
        canonical
    }

    pub(super) fn resolve_durable_owner(&mut self, path: &str) -> String {
        #[cfg(any(windows, unix))]
        if let Some(identity) = physical_file_identity(path) {
            let canonical = crate::dictionary::path::canonical_identity_key(path);
            let owner = self
                .owners
                .entry(identity)
                .or_insert_with(|| PhysicalOwner {
                    semantic_path: canonical,
                    durable_path: None,
                });
            return owner
                .durable_path
                .get_or_insert_with(|| path.to_string())
                .clone();
        }
        path.to_string()
    }

    pub(super) fn register_durable_owner(&mut self, path: &str) {
        let _ = self.resolve_durable_owner(path);
    }
}

impl super::McpState {
    /// Resolve the stable canonical path that owns semantic state. Existing
    /// hard-link spellings converge through platform physical-file identity.
    pub(crate) fn semantic_owner_path(&self, path: &str) -> String {
        lock_or_recover!(self.physical_identities.lock(), "physical_identities")
            .resolve_semantic_owner(path)
    }

    /// Resolve the first caller-shaped durable owner for a physical file.
    pub(crate) fn durable_owner_path(&self, path: &str) -> String {
        lock_or_recover!(self.physical_identities.lock(), "physical_identities")
            .resolve_durable_owner(path)
    }
}

#[cfg(windows)]
fn physical_file_identity(path: &str) -> Option<PhysicalFileIdentity> {
    let metadata = windows_physical_metadata(path)?;
    Some(PhysicalFileIdentity {
        filesystem_id: u64::from(metadata.volume_serial_number),
        file_id: metadata.file_index,
    })
}

#[cfg(unix)]
fn physical_file_identity(path: &str) -> Option<PhysicalFileIdentity> {
    use std::os::unix::fs::MetadataExt;

    let metadata = std::fs::metadata(path).ok()?;
    Some(PhysicalFileIdentity {
        filesystem_id: metadata.dev(),
        file_id: metadata.ino(),
    })
}

pub(crate) fn has_multiple_hard_links(path: &str) -> bool {
    #[cfg(windows)]
    {
        windows_physical_metadata(path).is_some_and(|metadata| metadata.link_count > 1)
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        std::fs::metadata(path).is_ok_and(|metadata| metadata.nlink() > 1)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = path;
        false
    }
}

#[cfg(windows)]
#[derive(Clone, Copy)]
struct WindowsPhysicalMetadata {
    volume_serial_number: u32,
    file_index: u64,
    link_count: u32,
}

#[cfg(windows)]
fn windows_physical_metadata(path: &str) -> Option<WindowsPhysicalMetadata> {
    use std::ffi::c_void;
    use std::mem::MaybeUninit;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetFileInformationByHandle"]
        fn get_file_information_by_handle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let file = std::fs::File::open(path).ok()?;
    let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
    // SAFETY: `file` owns a valid open Windows handle for the duration of the
    // call, and `information` points to writable storage of the exact Win32
    // `BY_HANDLE_FILE_INFORMATION` layout. The value is read only on success.
    let succeeded = unsafe {
        get_file_information_by_handle(file.as_raw_handle(), information.as_mut_ptr()) != 0
    };
    if !succeeded {
        return None;
    }
    // SAFETY: a successful `GetFileInformationByHandle` call initializes the
    // complete output structure.
    let information = unsafe { information.assume_init() };
    Some(WindowsPhysicalMetadata {
        volume_serial_number: information.volume_serial_number,
        file_index: (u64::from(information.file_index_high) << 32)
            | u64::from(information.file_index_low),
        link_count: information.number_of_links,
    })
}
