use super::super::model::EvidenceCategory;
use super::CargoCheckCompiler;
use super::json::CheckedJson;

#[derive(Clone, Copy)]
pub(crate) enum ProducerStream {
    Stdout,
    Stderr,
}

/// Only capture supplies this metadata; none is a tool argument. Byte length is
/// the observed payload, excluding newline, even when admission has stopped.
pub(crate) struct CapturedFrame<'a> {
    pub stream: ProducerStream,
    pub bytes: &'a [u8],
    pub observed_bytes: u64,
    pub terminated: bool,
    pub over_limit: bool,
    pub admission_cut: bool,
    pub decoding_available: bool,
}

impl CargoCheckCompiler {
    pub fn observe_stdout_frame(&mut self, bytes: &[u8], terminated: bool, over_limit: bool) {
        self.observe_direct_frame(ProducerStream::Stdout, bytes, terminated, over_limit);
    }

    pub fn observe_stderr_frame(&mut self, bytes: &[u8], terminated: bool, over_limit: bool) {
        self.observe_direct_frame(ProducerStream::Stderr, bytes, terminated, over_limit);
    }

    fn observe_direct_frame(
        &mut self,
        stream: ProducerStream,
        bytes: &[u8],
        terminated: bool,
        over_limit: bool,
    ) {
        let over_limit = over_limit || bytes.len() > self.policy.maximum_frame_bytes;
        self.observe_captured_frame(CapturedFrame {
            stream,
            bytes,
            observed_bytes: bytes.len() as u64,
            terminated,
            over_limit,
            admission_cut: false,
            decoding_available: !over_limit,
        });
    }

    pub(crate) fn observe_captured_frame(&mut self, frame: CapturedFrame<'_>) {
        let stdout = matches!(frame.stream, ProducerStream::Stdout);
        if stdout {
            self.coverage.stdout_frames += 1;
        } else {
            self.coverage.stderr_frames += 1;
        }
        let over_limit = frame.over_limit
            || frame.observed_bytes > self.policy.maximum_frame_bytes as u64
            || frame.bytes.len() > self.policy.maximum_frame_bytes;
        self.coverage.over_limit_frames += usize::from(over_limit);
        self.coverage.truncated_frames += usize::from(!frame.terminated);
        self.coverage.admission_cut_frames += usize::from(frame.admission_cut);
        let unavailable = !frame.decoding_available
            || over_limit
            || frame.bytes.len() as u64 != frame.observed_bytes;
        let decoded = if !unavailable {
            std::str::from_utf8(frame.bytes).ok()
        } else {
            None
        };
        self.coverage.decoding_unavailable_frames += usize::from(unavailable);
        if !unavailable && decoded.is_none() {
            self.coverage.invalid_utf8_frames += 1;
        }
        if over_limit || frame.admission_cut || !frame.terminated || decoded.is_none() {
            if stdout {
                self.coverage.unparsed_stdout_frames += 1;
                self.coverage.unparsed_stdout_bytes = self
                    .coverage
                    .unparsed_stdout_bytes
                    .saturating_add(frame.observed_bytes);
            } else {
                self.coverage.unparsed_stderr_frames += 1;
                self.coverage.unparsed_stderr_bytes = self
                    .coverage
                    .unparsed_stderr_bytes
                    .saturating_add(frame.observed_bytes);
            }
            let category = if stdout {
                EvidenceCategory::MalformedOrTruncated
            } else {
                EvidenceCategory::Stderr
            };
            self.evidence.observe_summary(
                category,
                "[producer frame withheld: framing, admission, or decoding fault]",
                source_bytes(frame.observed_bytes),
                &mut self.transformations,
            );
            return;
        }
        let text = decoded.expect("eligible frame has validated UTF-8");
        if text.trim().is_empty() {
            if stdout {
                self.coverage.empty_stdout_frames += 1;
            } else {
                self.coverage.empty_stderr_frames += 1;
            }
            return;
        }
        if !stdout {
            self.evidence
                .observe(EvidenceCategory::Stderr, text, &mut self.transformations);
            return;
        }
        let candidate = text.trim();
        if !candidate.starts_with('{') {
            self.coverage.non_json_stdout += 1;
            self.evidence.observe(
                EvidenceCategory::StdoutNonJson,
                candidate,
                &mut self.transformations,
            );
            return;
        }
        self.coverage.json_candidates += 1;
        let parsed = match serde_json::from_str::<CheckedJson>(candidate) {
            Ok(parsed) => parsed,
            Err(_) => {
                self.coverage.malformed_json += 1;
                self.evidence.observe_summary(
                    EvidenceCategory::MalformedOrTruncated,
                    "[malformed Cargo JSON candidate withheld]",
                    source_bytes(frame.observed_bytes),
                    &mut self.transformations,
                );
                return;
            }
        };
        self.coverage.parsed_json_objects += 1;
        if parsed.duplicate_fields != 0 {
            self.coverage.duplicate_json_fields += parsed.duplicate_fields;
            self.observe_incompatible(source_bytes(frame.observed_bytes));
            return;
        }
        self.observe_structured(parsed.value, source_bytes(frame.observed_bytes));
    }

    pub(crate) fn observe_stderr_terminal_sample(&mut self, bytes: &[u8]) {
        self.coverage.stderr_terminal_samples += 1;
        match std::str::from_utf8(bytes) {
            Ok(text) => self
                .evidence
                .observe_terminal_stderr(text, &mut self.transformations),
            Err(_) => {
                self.coverage.invalid_utf8_terminal_samples += 1;
                self.evidence.observe_terminal_stderr(
                    "[terminal stderr sample withheld: invalid UTF-8]",
                    &mut self.transformations,
                );
            }
        }
    }
}

fn source_bytes(bytes: u64) -> usize {
    bytes.min(usize::MAX as u64) as usize
}

#[cfg(test)]
#[path = "../../../tests/diagnostics/cargo_check_incremental.rs"]
mod tests;
