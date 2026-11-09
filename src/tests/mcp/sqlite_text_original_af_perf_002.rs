//! Frozen original metadata reader; observations do not affect decisions.
use crate::compression::Fidelity;
use crate::mcp::context_store::StoredContextMeta;
use crate::mcp::sqlite_store::SqliteStore;
use rusqlite::params;
pub(super) fn load_latest(
    store: &SqliteStore,
    file_path: &str,
) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>> {
    #[cfg(test)]
    crate::default_operation_observations_af_perf_002::metadata_query();
    let mut stmt = store.conn.prepare(
            "SELECT c.id, c.file_path, COALESCE(NULLIF(c.source_hash, ''), c.content_hash), c.fidelity, c.pretty_text, c.created_at,
                    COALESCE(c.raw_tokens, 0) as raw_tokens,
                    COALESCE(c.compressed_tokens, 0) as compressed_tokens,
                    (SELECT COUNT(*) FROM deltas d WHERE d.context_id = c.id) as delta_count
             FROM contexts c WHERE c.file_path = ?1 ORDER BY c.updated_at DESC LIMIT 1",
        )?;

    let mut rows = stmt.query(params![file_path])?;
    if let Some(row) = rows.next()? {
        let _id: String = row.get(0)?;
        let fp: String = row.get(1)?;
        let hash: String = row.get(2)?;
        let fid: i32 = row.get(3)?;
        #[cfg(test)]
        crate::default_operation_observations_af_perf_002::metadata_conversion();
        let _pretty: Option<String> = row.get(4)?;
        #[cfg(test)]
        crate::default_operation_observations_af_perf_002::metadata_text(_pretty.as_deref());
        let created_at_str: String = row.get(5)?;
        let raw_tokens: i64 = row.get(6)?;
        let compressed_tokens: i64 = row.get(7)?;
        let delta_count: i32 = row.get(8)?;

        // Parse Fidelity from i32
        let fidelity = match fid {
            0 => Fidelity::Low,
            1 => Fidelity::Medium,
            2 => Fidelity::High,
            3 => Fidelity::Edit,
            4 => Fidelity::Verbatim,
            _ => Fidelity::Low,
        };

        // HIGH-01 fix: compute version from delta count
        let version = (delta_count as u64) + 1;

        // HIGH-02 fix: parse actual created_at from DB
        let created_at = super::super::super::chrono_parse_or_now(&created_at_str);

        Ok(Some(StoredContextMeta {
            file_path: fp,
            fidelity,
            version,
            is_angular: false,
            source_hash: hash,
            created_at,
            raw_tokens: raw_tokens as u64,
            compressed_tokens: compressed_tokens as u64,
        }))
    } else {
        Ok(None)
    }
}
