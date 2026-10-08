use super::super::{CargoCheckCompiler, CargoCheckPolicy};
use super::StreamCaptureFacts;
use std::io::{self, Read};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub(super) struct Framer {
    stdout: bool,
    policy: CargoCheckPolicy,
    limit: u64,
    facts: StreamCaptureFacts,
    frame: Vec<u8>,
    frame_bytes: u64,
    frame_cut: bool,
    late_stderr_frame: Vec<u8>,
    last_stderr_tail: Option<Vec<u8>>,
}

impl Framer {
    pub fn new(stdout: bool, policy: CargoCheckPolicy) -> Self {
        let limit = if stdout {
            policy.stdout_capture_bytes
        } else {
            policy.stderr_capture_bytes
        };
        Self {
            stdout,
            policy,
            limit: limit as u64,
            facts: StreamCaptureFacts::default(),
            frame: Vec::new(),
            frame_bytes: 0,
            frame_cut: false,
            late_stderr_frame: Vec::new(),
            last_stderr_tail: None,
        }
    }

    pub fn observe(&mut self, bytes: &[u8], compiler: &mut CargoCheckCompiler) {
        for &byte in bytes {
            self.facts.observed_bytes = self.facts.observed_bytes.saturating_add(1);
            let admitted = self.facts.admitted_bytes < self.limit;
            if admitted {
                self.facts.admitted_bytes += 1;
            } else {
                self.facts.discarded_bytes = self.facts.discarded_bytes.saturating_add(1);
                self.facts.capture_limit_activated = true;
                self.frame_cut = true;
            }
            if byte == b'\n' {
                self.flush(true, compiler);
                continue;
            }
            self.frame_bytes = self.frame_bytes.saturating_add(1);
            if admitted && self.frame.len() < self.policy.maximum_frame_bytes {
                self.frame.push(byte);
            }
            // A separate complete-line sample protects stderr's terminal cause
            // after admission stops. Oversized or partial samples are withheld,
            // not sliced across a potential secret and then treated as complete.
            if !self.stdout && self.late_stderr_frame.len() < self.policy.evidence_bytes {
                self.late_stderr_frame.push(byte);
            }
        }
    }

    fn flush(&mut self, terminated: bool, compiler: &mut CargoCheckCompiler) {
        self.facts.frames_observed = self.facts.frames_observed.saturating_add(1);
        self.facts.complete_frames += u64::from(terminated);
        self.facts.unterminated_frames += u64::from(!terminated);
        self.facts.maximum_observed_frame_bytes = self
            .facts
            .maximum_observed_frame_bytes
            .max(self.frame_bytes);
        let over_limit = self.frame_bytes > self.policy.maximum_frame_bytes as u64;
        self.facts.over_limit_frames += u64::from(over_limit);
        self.facts.admission_cut_frames += u64::from(self.frame_cut);
        // Empty frames count as framing evidence without creating unbounded
        // zero-byte evidence items in the existing compiler's collector.
        if !self.frame.is_empty() {
            // Incomplete samples may cut through a multiline credential. Do
            // not pass an arbitrary raw prefix to a whole-string redactor.
            let bytes = if over_limit || self.frame_cut || !terminated {
                b"[producer frame withheld: incomplete or over limit]".as_slice()
            } else {
                &self.frame
            };
            if self.stdout {
                compiler.observe_stdout_frame(bytes, terminated && !self.frame_cut, over_limit);
            } else {
                compiler.observe_stderr_frame(bytes, terminated && !self.frame_cut, over_limit);
            }
        }
        if !self.stdout && self.frame_cut {
            if terminated && self.frame_bytes == 0 {
                // Empty delimiters carry no terminal evidence; do not evict
                // the most recent eligible nonempty sample.
            } else if terminated && self.frame_bytes <= self.policy.evidence_bytes as u64 {
                self.facts.post_limit_stderr_samples_replaced +=
                    u64::from(self.last_stderr_tail.is_some());
                self.last_stderr_tail = Some(std::mem::take(&mut self.late_stderr_frame));
                self.facts.post_limit_stderr_frame_candidates += 1;
            } else {
                self.facts.post_limit_stderr_frames_withheld += 1;
            }
        }
        self.frame.clear();
        self.late_stderr_frame.clear();
        self.frame_bytes = 0;
        self.frame_cut = false;
    }

    pub fn finish(mut self, compiler: &mut CargoCheckCompiler) -> StreamCaptureFacts {
        if self.frame_bytes != 0 {
            self.flush(false, compiler);
        }
        if let Some(tail) = self.last_stderr_tail {
            compiler.observe_stderr_terminal_sample(&tail);
        }
        self.facts
    }
}

pub(super) fn drain<R: Read>(
    mut pipe: R,
    stdout: bool,
    compiler: &Mutex<CargoCheckCompiler>,
    stop: &AtomicBool,
) -> StreamCaptureFacts {
    let mut framer = Framer::new(stdout, CargoCheckPolicy::APPROVED);
    let mut buffer = [0u8; 8192];
    loop {
        if stop.load(Ordering::Acquire) {
            framer.facts.stopped_before_eof = true;
            break;
        }
        match pipe.read(&mut buffer) {
            Ok(0) => {
                framer.facts.eof_observed = true;
                break;
            }
            Ok(length) => framer.observe(
                &buffer[..length],
                &mut compiler.lock().unwrap_or_else(|error| error.into_inner()),
            ),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => {
                framer.facts.read_failed = true;
                framer.facts.read_error_os_code = error.raw_os_error();
                break;
            }
        }
    }
    framer.finish(&mut compiler.lock().unwrap_or_else(|error| error.into_inner()))
}

#[cfg(test)]
#[path = "../../../tests/diagnostics/cargo_check_capture.rs"]
mod tests;
