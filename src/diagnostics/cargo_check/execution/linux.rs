use super::super::{CargoCheckInvocation, ExecutionError, ProcessOutcome};
use nix::errno::Errno;
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use nix::sys::signal::{Signal, killpg};
use nix::sys::wait::{Id, WaitPidFlag, WaitStatus, waitid};
use nix::unistd::Pid;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

pub(crate) struct Pipe(File);

impl Pipe {
    fn new(fd: OwnedFd) -> io::Result<Self> {
        let flags = OFlag::from_bits_truncate(fcntl(&fd, FcntlArg::F_GETFL)?);
        fcntl(&fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;
        Ok(Self(File::from(fd)))
    }
}

impl Read for Pipe {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.0.read(bytes)
    }
}

pub(crate) struct OwnedProcess {
    child: Child,
    group: Pid,
    finished: bool,
}

impl OwnedProcess {
    pub const MECHANISM: &'static str = "linux_process_group";
    pub const FORCED_MECHANISM: &'static str = "SIGKILL_process_group";

    pub fn start(invocation: &CargoCheckInvocation) -> Result<(Self, Pipe, Pipe), ExecutionError> {
        // std establishes the dedicated process group in the child before exec.
        let mut command = Command::new(invocation.executable());
        command
            .args(invocation.arguments())
            .current_dir(invocation.current_directory())
            .env_clear()
            .envs(invocation.environment())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        invocation.revalidate()?;
        let child = command.spawn()?;
        let group = Pid::from_raw(child.id() as i32);
        let mut owner = Self {
            child,
            group,
            finished: false,
        };
        let stdout = owner
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::from(io::ErrorKind::BrokenPipe))?;
        let stderr = owner
            .child
            .stderr
            .take()
            .ok_or_else(|| io::Error::from(io::ErrorKind::BrokenPipe))?;
        Ok((owner, Pipe::new(stdout.into())?, Pipe::new(stderr.into())?))
    }

    pub fn poll(&mut self) -> io::Result<Option<ProcessOutcome>> {
        // WNOWAIT pins the leader PID until cleanup ends, preventing group-ID
        // reuse between root exit observation and a later group cancellation.
        let status = waitid(
            Id::Pid(self.group),
            WaitPidFlag::WEXITED | WaitPidFlag::WNOHANG | WaitPidFlag::WNOWAIT,
        )?;
        Ok(match status {
            WaitStatus::Exited(_, code) => Some(ProcessOutcome {
                exit_code: Some(i64::from(code)),
                signal: None,
            }),
            WaitStatus::Signaled(_, signal, _) => Some(ProcessOutcome {
                exit_code: None,
                signal: Some(signal as i32),
            }),
            _ => None,
        })
    }

    pub fn quiescent(&self) -> io::Result<bool> {
        // kill(group, 0) includes zombies. Zombies cannot execute or keep pipes
        // open; /proc distinguishes them from live owned descendants.
        for entry in std::fs::read_dir("/proc")? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().parse::<u32>().is_err() {
                continue;
            }
            let stat = match std::fs::read_to_string(entry.path().join("stat")) {
                Ok(stat) => stat,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            let tail = stat.rsplit_once(')').ok_or(io::ErrorKind::InvalidData)?.1;
            let mut fields = tail.split_whitespace();
            let state = fields.next().ok_or(io::ErrorKind::InvalidData)?;
            let _parent = fields.next();
            let group = fields
                .next()
                .and_then(|field| field.parse::<i32>().ok())
                .ok_or(io::ErrorKind::InvalidData)?;
            if group == self.group.as_raw() && !matches!(state, "Z" | "X") {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn terminate(&self) -> io::Result<()> {
        match killpg(self.group, Signal::SIGKILL) {
            Ok(()) | Err(Errno::ESRCH) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.child.wait()?;
        self.finished = true;
        Ok(())
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.terminate();
            let _ = self.child.wait();
        }
    }
}
