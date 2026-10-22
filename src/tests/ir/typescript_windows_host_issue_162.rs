use super::{Host, Witness};
use std::ffi::c_void;
use std::fs::OpenOptions;
use std::os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle};
use std::path::{Path, PathBuf};

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "DeviceIoControl"]
    fn control(
        handle: *mut c_void,
        code: u32,
        input: *const u8,
        input_size: u32,
        output: *mut u8,
        output_size: u32,
        returned: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
}

pub(super) struct Junction(PathBuf);
impl Drop for Junction {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.0).expect("remove only the test junction");
    }
}

pub(super) fn junction(link: &Path, target: &Path) -> Junction {
    let canonical = target.canonicalize().unwrap();
    let text = canonical.to_str().unwrap();
    let substitute = format!("\\??\\{}", text.strip_prefix("\\\\?\\").unwrap_or(text));
    let substitute: Vec<u16> = substitute.encode_utf16().collect();
    let print: Vec<u16> = target.as_os_str().encode_wide().collect();
    let mut data = Vec::new();
    data.extend_from_slice(&0xA0000003_u32.to_le_bytes()); // IO_REPARSE_TAG_MOUNT_POINT
    let length = 8 + 2 * (substitute.len() + print.len() + 2);
    data.extend_from_slice(&u16::try_from(length).unwrap().to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    for value in [
        0,
        substitute.len() * 2,
        (substitute.len() + 1) * 2,
        print.len() * 2,
    ] {
        data.extend_from_slice(&u16::try_from(value).unwrap().to_le_bytes());
    }
    for value in substitute.into_iter().chain([0]).chain(print).chain([0]) {
        data.extend_from_slice(&value.to_le_bytes());
    }
    std::fs::create_dir(link).unwrap();
    let file = OpenOptions::new()
        .access_mode(0x40000000)
        .custom_flags(0x02200000)
        .open(link)
        .unwrap(); // reparse point + backup semantics
    let mut returned = 0;
    // SAFETY: the open handle is live; data has the exact mount-point reparse
    // layout above, input_size is its byte length, and output/overlapped are null.
    let result = unsafe {
        control(
            file.as_raw_handle(),
            0x000900A4,
            data.as_ptr(),
            u32::try_from(data.len()).unwrap(),
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(result, 0, "{}", std::io::Error::last_os_error());
    Junction(link.to_path_buf())
}

#[test]
fn issue_162_windows_junction_escape_does_not_consume_foreign_bytes() {
    let root = tempfile::tempdir().unwrap();
    let foreign = tempfile::tempdir().unwrap();
    std::fs::write(foreign.path().join("leaf.ts"), "export class Foreign {}").unwrap();
    let link = root.path().join("escape");
    let _junction = junction(&link, foreign.path());
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(&link.join("leaf.ts")).is_none());
    assert!(!host.current());
    assert!(
        !host
            .witnesses
            .iter()
            .any(|w| matches!(w, Witness::Content { .. }))
    );
}

#[test]
fn issue_162_windows_junction_retarget_invalidates_consumed_route() {
    let root = tempfile::tempdir().unwrap();
    for (directory, source) in [
        ("one", "export class One {}"),
        ("two", "export class Two {}"),
    ] {
        std::fs::create_dir(root.path().join(directory)).unwrap();
        std::fs::write(root.path().join(directory).join("leaf.ts"), source).unwrap();
    }
    let link = root.path().join("route");
    let first = junction(&link, &root.path().join("one"));
    let mut before = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        &*before.read(&link.join("leaf.ts")).unwrap(),
        "export class One {}"
    );
    assert!(before.current());
    drop(first);
    let _second = junction(&link, &root.path().join("two"));
    assert!(!before.current());
    let mut fresh = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        &*fresh.read(&link.join("leaf.ts")).unwrap(),
        "export class Two {}"
    );
    assert!(fresh.current());
}

#[test]
fn issue_162_windows_sharing_denial_is_not_missing_source() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("leaf.ts");
    std::fs::write(&file, "export class Leaf {}").unwrap();
    let exclusive = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&file)
        .unwrap();
    let mut host = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(host.read(&file).is_none());
    assert!(!host.current());
    assert!(
        host.witnesses
            .iter()
            .any(|w| matches!(w, Witness::Unreadable { .. }))
    );
    assert!(
        !host
            .witnesses
            .iter()
            .any(|w| matches!(w, Witness::Missing { .. }))
    );
    drop(exclusive);
    let mut fresh = Host::new(&[root.path().to_path_buf()]).unwrap();
    assert!(fresh.read(&file).is_some());
}

#[test]
fn issue_162_windows_capture_junction_keeps_route_and_canonical_declaration_separate() {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("real");
    std::fs::create_dir(&real).unwrap();
    std::fs::write(real.join("app.ts"), "export class App {}").unwrap();
    let route = root.path().join("alias");
    let _junction = junction(&route, &real);
    let evidence = super::TypeScriptAuthority::capture(
        &route.join("app.ts"),
        "export class App {}",
        &[root.path().to_path_buf()],
    );
    let expected = Host::new(&[root.path().to_path_buf()])
        .unwrap()
        .normalized(&real.join("app.ts"))
        .unwrap();
    assert_eq!(evidence.source_route, route.join("app.ts"));
    assert_eq!(evidence.source, expected);
    assert_eq!(evidence.declarations[0].source, expected);
    assert!(
        evidence
            .references
            .iter()
            .any(|(name, _, resolution)| name == "App"
                && matches!(resolution, super::Resolution::Resolved(id) if id.source == expected))
    );
}
