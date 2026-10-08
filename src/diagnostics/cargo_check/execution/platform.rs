#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod implementation;
#[cfg(windows)]
#[path = "windows.rs"]
mod implementation;

#[cfg(any(target_os = "linux", windows))]
pub(super) use implementation::OwnedProcess;

#[cfg(not(any(target_os = "linux", windows)))]
mod implementation {
    use super::super::{CargoCheckInvocation, ExecutionError, ProcessOutcome};
    use std::io;

    pub struct Pipe;
    impl io::Read for Pipe {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::Unsupported.into())
        }
    }
    pub struct OwnedProcess;
    impl OwnedProcess {
        pub fn start(_: &CargoCheckInvocation) -> Result<(Self, Pipe, Pipe), ExecutionError> {
            Err(ExecutionError::UnsupportedPlatform)
        }
        pub fn poll(&mut self) -> io::Result<Option<ProcessOutcome>> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub fn quiescent(&self) -> io::Result<bool> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub fn terminate(&self) -> io::Result<()> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub fn finish(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub const MECHANISM: &'static str = "unsupported";
        pub const FORCED_MECHANISM: &'static str = "unsupported";
    }
}
#[cfg(not(any(target_os = "linux", windows)))]
pub(super) use implementation::OwnedProcess;
