//! Request-bound host input quanta for opt-in direct evidence only.
use super::Host;
use super::source_observations::request_budget::{Cost, Work};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

impl Host {
    pub(super) fn direct_work(&mut self, kind: Work, path: &Path) -> std::io::Result<()> {
        let Some(request) = &self.direct_budget else {
            return Ok(());
        };
        if !request.is_integrated() {
            return Ok(());
        }
        let result = (|| {
            request.work(kind, false)?;
            let bytes = (path.as_os_str().len() as u64)
                .checked_mul(8)
                .and_then(|v| v.checked_add(512))
                .ok_or(super::source_observations::request_budget::Stop::RetainedBytes)?;
            request.reserve_input(Cost {
                retained_bytes: bytes,
                ..Cost::default()
            })
        })();
        match result {
            Ok(permit) => {
                self.direct_reservations.push(permit.finish());
                Ok(())
            }
            Err(stop) => {
                self.direct_failure = Some(stop);
                Err(std::io::Error::other(format!("host budget: {stop:?}")))
            }
        }
    }
    pub(super) fn read_direct_bytes(
        &mut self,
        path: &Path,
        file: std::fs::File,
    ) -> std::io::Result<Vec<u8>> {
        self.direct_work(Work::Metadata, path)?;
        let request = self.direct_budget.as_ref().expect("request-bound host");
        let expected = file.metadata()?.len();
        let bounded = expected
            .checked_add(1)
            .ok_or_else(|| std::io::Error::other("source size overflow"))?;
        let permit = request
            .reserve_input(Cost {
                source_bytes: if self.direct_source { bounded } else { 0 },
                support_bytes: if self.direct_source { 0 } else { bounded },
                retained_bytes: bounded
                    .checked_mul(if request.is_integrated() { 16 } else { 2 })
                    .ok_or_else(|| std::io::Error::other("source size overflow"))?,
                ..Cost::default()
            })
            .map_err(|e| {
                self.direct_failure = Some(e);
                std::io::Error::other(format!("direct input budget: {e:?}"))
            })?;
        let mut bytes =
            Vec::with_capacity(usize::try_from(bounded).map_err(std::io::Error::other)?);
        let mut reader = file.take(bounded);
        let mut chunk = [0u8; 4096];
        loop {
            request.checkpoint().map_err(|e| {
                self.direct_failure = Some(e);
                std::io::Error::other(format!("read budget: {e:?}"))
            })?;
            let n = reader.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..n]);
        }
        #[cfg(test)]
        super::e1_metrics::read(path, bytes.len(), false);
        #[cfg(feature = "resolver_evidence_v2")]
        super::source_observations::request_budget::telemetry::read(bytes.len(), false);
        #[cfg(not(test))]
        let _ = path;
        if bytes.len() as u64 > expected {
            return Err(std::io::Error::other("direct input grew during read"));
        }
        self.direct_reservations.push(permit.finish());
        Ok(bytes)
    }
    pub(super) fn direct_entries(&mut self, path: &Path) -> Result<Vec<PathBuf>, String> {
        self.direct_work(Work::Directory, path)
            .map_err(|e| e.to_string())?;
        let request = self.direct_budget.as_ref().expect("request-bound host");
        let mut entries = Vec::new();
        let mut directory = std::fs::read_dir(path).map_err(|e| e.to_string())?;
        loop {
            request.work(Work::DirectoryEntry, false).map_err(|e| {
                self.direct_failure = Some(e);
                format!("directory budget: {e:?}")
            })?;
            request.checkpoint().map_err(|e| {
                self.direct_failure = Some(e);
                format!("direct directory budget: {e:?}")
            })?;
            let Some(entry) = directory.next() else {
                break;
            };
            let path = entry.map_err(|e| e.to_string())?.path();
            let bytes = (path.as_os_str().len() as u64)
                .saturating_mul(4)
                .saturating_add(128);
            let permit = request
                .reserve_input(Cost {
                    edges: 1,
                    retained_bytes: bytes,
                    ..Cost::default()
                })
                .map_err(|e| {
                    self.direct_failure = Some(e);
                    format!("direct directory budget: {e:?}")
                })?;
            entries.push(path);
            self.direct_reservations.push(permit.finish());
        }
        entries.sort();
        Ok(entries)
    }
}
