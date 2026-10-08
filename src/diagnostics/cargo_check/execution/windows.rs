//! Win32 calls are confined to this ownership boundary. Every returned handle
//! is immediately owned; Cargo remains suspended until Job assignment succeeds.
use super::super::{CargoCheckInvocation, ExecutionError, ProcessOutcome};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read};
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::JobObjects::*;
use windows_sys::Win32::System::Pipes::{CreatePipe, PeekNamedPipe};
use windows_sys::Win32::System::Threading::*;

pub(crate) struct Pipe(File);

impl Read for Pipe {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let mut available = 0;
        // SAFETY: the File owns a valid pipe handle; only this thread reads it.
        let ok = unsafe {
            PeekNamedPipe(
                self.0.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        };
        if ok == 0 {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                Ok(0)
            } else {
                Err(error)
            };
        }
        if available == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let length = bytes.len().min(available as usize);
        self.0.read(&mut bytes[..length])
    }
}

struct AttributeList {
    storage: Vec<usize>,
}

impl AttributeList {
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }

    fn handles(handles: &[HANDLE; 3]) -> io::Result<Self> {
        let mut size = 0;
        // SAFETY: the documented sizing call uses a null list and writable size.
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size);
        }
        if size == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut storage = vec![0usize; size.div_ceil(size_of::<usize>())];
        // SAFETY: storage has sufficient size/alignment and remains at this
        // allocation until DeleteProcThreadAttributeList is called.
        if unsafe {
            InitializeProcThreadAttributeList(storage.as_mut_ptr().cast(), 1, 0, &mut size)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut list = Self { storage };
        // SAFETY: exactly the three live, inheritable stdio handles are supplied;
        // the borrowed array remains live through the CreateProcessW call.
        if unsafe {
            UpdateProcThreadAttribute(
                list.pointer(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                size_of::<[HANDLE; 3]>(),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(list)
    }
}

impl Drop for AttributeList {
    fn drop(&mut self) {
        // SAFETY: this is the successfully initialized list owned by storage.
        unsafe {
            DeleteProcThreadAttributeList(self.pointer());
        }
    }
}

fn pipe() -> io::Result<(OwnedHandle, OwnedHandle)> {
    let mut read = null_mut();
    let mut write = null_mut();
    let security = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    // SAFETY: writable outputs and a correctly sized security structure.
    if unsafe { CreatePipe(&mut read, &mut write, &security, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: both successful CreatePipe outputs are exclusively transferred.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read),
            OwnedHandle::from_raw_handle(write),
        )
    })
}

fn non_inherited(handle: &OwnedHandle) -> io::Result<()> {
    // SAFETY: the supplied handle is live and owned throughout this call.
    if unsafe { SetHandleInformation(handle.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    value.push(0);
    Ok(value)
}

pub(crate) struct OwnedProcess {
    job: OwnedHandle,
    root: OwnedHandle,
    assigned: bool,
    finished: bool,
}

impl OwnedProcess {
    pub const MECHANISM: &'static str = "windows_job_object";
    pub const FORCED_MECHANISM: &'static str = "TerminateJobObject";

    pub fn start(invocation: &CargoCheckInvocation) -> Result<(Self, Pipe, Pipe), ExecutionError> {
        let application = wide(invocation.executable().as_os_str())?;
        if application.contains(&(b'"' as u16)) {
            return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
        }
        // Native CreateProcessW accepts a command line. Only the approved path
        // and the two fixed arguments are encoded; no shell or caller args.
        let mut command = vec![b'"' as u16];
        command.extend_from_slice(&application[..application.len() - 1]);
        command.extend("\" check --message-format=json".encode_utf16());
        command.push(0);
        let directory = wide(invocation.current_directory().as_os_str())?;
        let mut environment = Vec::new();
        let mut entries: Vec<_> = invocation.environment().iter().collect();
        entries.sort_by_key(|(name, _)| name.to_string_lossy().to_ascii_uppercase());
        for (name, value) in entries {
            let name = wide(name)?;
            if name.contains(&(b'=' as u16)) {
                return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
            }
            environment.extend_from_slice(&name[..name.len() - 1]);
            environment.push(b'=' as u16);
            environment.extend(wide(value)?);
        }
        environment.push(0);
        // SAFETY: null security/name requests a private, non-inherited Job.
        let job = unsafe { CreateJobObjectW(null(), null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error().into());
        }
        // SAFETY: successful API output is exclusively owned here.
        let job = unsafe { OwnedHandle::from_raw_handle(job) };
        // SAFETY: Win32 structures document zero initialization.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // No breakaway flags are granted.
        // SAFETY: live Job and correctly sized limits structure.
        if unsafe {
            SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error().into());
        }
        let (stdin, stdin_writer) = pipe()?;
        drop(stdin_writer); // the child receives inert EOF, never host input
        let (stdout, stdout_writer) = pipe()?;
        let (stderr, stderr_writer) = pipe()?;
        non_inherited(&stdout)?;
        non_inherited(&stderr)?;
        let inherited = [
            stdin.as_raw_handle(),
            stdout_writer.as_raw_handle(),
            stderr_writer.as_raw_handle(),
        ];
        let mut attributes = AttributeList::handles(&inherited)?;
        // SAFETY: structures document zero initialization; cb selects EX layout.
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = inherited[0];
        startup.StartupInfo.hStdOutput = inherited[1];
        startup.StartupInfo.hStdError = inherited[2];
        startup.lpAttributeList = attributes.pointer();
        // SAFETY: the writable output structure has the documented initial form.
        let mut process: PROCESS_INFORMATION = unsafe { zeroed() };
        invocation.revalidate()?;
        // SAFETY: all pointers refer to live, terminated UTF-16 buffers or
        // initialized structures. The handle list restricts inheritance; the
        // suspended flag prevents any producer work before Job assignment.
        if unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                1,
                CREATE_SUSPENDED
                    | CREATE_NO_WINDOW
                    | CREATE_UNICODE_ENVIRONMENT
                    | EXTENDED_STARTUPINFO_PRESENT,
                environment.as_ptr().cast(),
                directory.as_ptr(),
                &startup.StartupInfo,
                &mut process,
            )
        } == 0
        {
            return Err(io::Error::last_os_error().into());
        }
        // SAFETY: successful process and thread handles are exclusively owned.
        let root = unsafe { OwnedHandle::from_raw_handle(process.hProcess) };
        let thread = unsafe { OwnedHandle::from_raw_handle(process.hThread) };
        let mut owner = Self {
            job,
            root,
            assigned: false,
            finished: false,
        };
        // SAFETY: both handles remain live and Cargo is still suspended.
        if unsafe {
            AssignProcessToJobObject(owner.job.as_raw_handle(), owner.root.as_raw_handle())
        } == 0
        {
            return Err(io::Error::last_os_error().into());
        }
        owner.assigned = true;
        // SAFETY: assignment succeeded; this is the owned primary thread.
        if unsafe { ResumeThread(thread.as_raw_handle()) } == u32::MAX {
            return Err(io::Error::last_os_error().into());
        }
        Ok((owner, Pipe(File::from(stdout)), Pipe(File::from(stderr))))
    }

    pub fn poll(&mut self) -> io::Result<Option<ProcessOutcome>> {
        // SAFETY: the process handle remains live throughout observation.
        match unsafe { WaitForSingleObject(self.root.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0;
                // SAFETY: live root handle and writable exit-code output.
                if unsafe { GetExitCodeProcess(self.root.as_raw_handle(), &mut code) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(Some(ProcessOutcome {
                    exit_code: Some(i64::from(code)),
                    signal: None,
                }))
            }
            _ => Err(io::Error::last_os_error()),
        }
    }

    pub fn quiescent(&self) -> io::Result<bool> {
        // SAFETY: this API writes a fixed-size accounting structure.
        let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        // SAFETY: live Job and correctly sized writable accounting buffer.
        if unsafe {
            QueryInformationJobObject(
                self.job.as_raw_handle(),
                JobObjectBasicAccountingInformation,
                (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(accounting.ActiveProcesses == 0)
    }

    pub fn terminate(&self) -> io::Result<()> {
        // SAFETY: owned live Job; unassigned suspended roots are killed directly.
        let result = unsafe {
            if self.assigned {
                TerminateJobObject(self.job.as_raw_handle(), 1)
            } else {
                TerminateProcess(self.root.as_raw_handle(), 1)
            }
        };
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.finished = true;
        Ok(())
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.terminate();
        }
        // OwnedHandles close deterministically; Job close kills any residual
        // members even if explicit termination failed.
    }
}
