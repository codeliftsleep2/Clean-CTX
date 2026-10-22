//! Validate the actual opened file before consuming bytes through its handle.
use std::ffi::{OsString, c_void};
use std::fs::File;
use std::io;
use std::os::windows::{ffi::OsStringExt, io::AsRawHandle};
use std::path::PathBuf;

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "GetFinalPathNameByHandleW"]
    fn final_path(handle: *mut c_void, buffer: *mut u16, capacity: u32, flags: u32) -> u32;
}

pub(super) fn opened_path(file: &File) -> io::Result<PathBuf> {
    let mut buffer = Vec::<u16>::new();
    loop {
        let capacity = u32::try_from(buffer.len())
            .map_err(|_| io::Error::other("Windows final path does not fit the API length"))?;
        let output = if capacity == 0 {
            std::ptr::null_mut()
        } else {
            buffer.as_mut_ptr()
        };
        // SAFETY: file owns a live handle. For capacity zero Win32 only returns
        // the required length; otherwise buffer owns capacity writable UTF-16s.
        let length = unsafe { final_path(file.as_raw_handle(), output, capacity, 0) };
        if length == 0 {
            return Err(io::Error::last_os_error());
        }
        if length < capacity {
            buffer.truncate(length as usize);
            return Ok(PathBuf::from(OsString::from_wide(&buffer)));
        }
        buffer.resize(length as usize, 0);
    }
}
