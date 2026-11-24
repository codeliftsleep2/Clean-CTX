//! Explicit operator ceilings and caller requests; no production default values.
use super::{Cost, Error, Limits};
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Finalization {
    pub elapsed_ms: u64,
    pub validation_bytes: u64,
    pub validation_checks: u64,
    pub response_bytes: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Budget {
    pub elapsed_ms: u64,
    pub source_bytes: u64,
    pub support_bytes: u64,
    pub retained_bytes: u64,
    pub active_expensive_tasks: u64,
    pub traversal_units: u64,
    pub traversal_edges: u64,
    pub validation_bytes: u64,
    pub validation_checks: u64,
    pub response_bytes: u64,
    pub finalization: Finalization,
}
impl Budget {
    pub fn parse(v: &serde_json::Value) -> Result<Self, Error> {
        let b: Self =
            serde_json::from_value(v.clone()).map_err(|e| Error::invalid(e.to_string()))?;
        b.limits()?;
        Ok(b)
    }
    pub fn limits(&self) -> Result<Limits, Error> {
        let total = Cost {
            source_bytes: self.source_bytes,
            support_bytes: self.support_bytes,
            retained_bytes: self.retained_bytes,
            units: self.traversal_units,
            edges: self.traversal_edges,
            validation_bytes: self.validation_bytes,
            validation_checks: self.validation_checks,
            response_bytes: self.response_bytes,
        };
        if self.elapsed_ms == 0
            || self.active_expensive_tasks == 0
            || self.finalization.elapsed_ms > self.elapsed_ms
            || self.finalization.validation_bytes > self.validation_bytes
            || self.finalization.validation_checks > self.validation_checks
            || self.finalization.response_bytes > self.response_bytes
            || self.source_bytes.checked_add(self.support_bytes).is_none()
            || std::time::Instant::now()
                .checked_add(Duration::from_millis(self.elapsed_ms))
                .is_none()
        {
            return Err(Error::invalid(
                "Invalid or unrepresentable resource allowance",
            ));
        }
        for v in [
            self.source_bytes,
            self.support_bytes,
            self.retained_bytes,
            self.traversal_units,
            self.traversal_edges,
            self.validation_bytes,
            self.validation_checks,
            self.response_bytes,
        ] {
            usize::try_from(v).map_err(|_| Error::invalid("Unrepresentable capacity"))?;
        }
        Ok(Limits {
            total,
            finalization: Cost {
                validation_bytes: self.finalization.validation_bytes,
                validation_checks: self.finalization.validation_checks,
                response_bytes: self.finalization.response_bytes,
                ..Cost::default()
            },
            deadline: Duration::from_millis(self.elapsed_ms),
            finalization_time: Duration::from_millis(self.finalization.elapsed_ms),
            active_tasks: usize::try_from(self.active_expensive_tasks)
                .map_err(|_| Error::invalid("Unrepresentable concurrency"))?,
        })
    }
    pub fn capped(&self, ceiling: &Self) -> Result<Self, Error> {
        let mut v = serde_json::to_value(self).expect("budget");
        let cap = serde_json::to_value(ceiling).expect("budget");
        for (key, value) in v.as_object_mut().expect("object") {
            if key == "finalization" {
                for (key, n) in value.as_object_mut().expect("finalization") {
                    *n = serde_json::json!(
                        n.as_u64()
                            .expect("integer")
                            .min(cap["finalization"][key].as_u64().expect("integer"))
                    );
                }
            } else {
                *value = serde_json::json!(
                    value
                        .as_u64()
                        .expect("integer")
                        .min(cap[key].as_u64().expect("integer"))
                );
            }
        }
        // The public adapter executes source/support work sequentially per request.
        v["active_expensive_tasks"] = serde_json::json!(1);
        for (part, total) in [
            ("elapsed_ms", "elapsed_ms"),
            ("validation_bytes", "validation_bytes"),
            ("validation_checks", "validation_checks"),
            ("response_bytes", "response_bytes"),
        ] {
            v["finalization"][part] = serde_json::json!(
                v["finalization"][part]
                    .as_u64()
                    .unwrap()
                    .min(v[total].as_u64().unwrap())
            );
        }
        Self::parse(&v)
    }
}
