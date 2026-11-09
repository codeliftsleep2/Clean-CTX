//! Validate discarded presentation text with String's SQL/UTF-8 conversion contract.
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};

pub(super) struct ValidatedText;

impl FromSql for ValidatedText {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = value.as_str()?;
        #[cfg(test)]
        crate::default_operation_observations_af_perf_002::metadata_validated_bytes(text.len());
        // Retain UTF-8 validation in release builds without retaining text ownership.
        let _ = text;
        Ok(Self)
    }
}
