//! Symmetric test-only query/projection forms; production is never delegated here.
use crate::mcp::sqlite_store::{PersistedContextSummary, SqliteStore};
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};

#[derive(Default, Debug, serde::Serialize)]
pub(super) struct Work {
    pub queries: usize,
    pub rows: usize,
    pub conversions: usize,
    pub summaries: usize,
    pub paths: usize,
    pub owned_strings: usize,
    pub owned_bytes: usize,
    pub validated_timestamp_bytes: usize,
    pub count_results: usize,
    pub delta_total: i64,
    pub capacity: usize,
    pub vm_steps: i32,
    pub fullscan_steps: i32,
}
struct TextLength(usize);
impl FromSql for TextLength {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(|v| Self(v.len()))
    }
}
pub(super) enum Output {
    Summaries(Vec<PersistedContextSummary>),
    Paths(Vec<String>),
}
impl Output {
    pub fn paths(&self) -> Vec<&str> {
        match self {
            Self::Summaries(v) => v.iter().map(|v| v.file_path.as_str()).collect(),
            Self::Paths(v) => v.iter().map(String::as_str).collect(),
        }
    }
}
pub(super) fn sql() -> &'static str {
    let source = include_str!("../../mcp/sqlite_store.rs");
    let begin = source.find("pub fn list_contexts(").unwrap();
    let begin = source[begin..].find("\"SELECT ").unwrap() + begin + 1;
    let end = source[begin..].find("\",").unwrap() + begin;
    &source[begin..end]
}
pub(super) fn read(
    store: &SqliteStore,
    projected: bool,
    observe: bool,
) -> rusqlite::Result<(Output, Work)> {
    let mut work = Work::default();
    if observe {
        work.queries += 1;
    }
    let mut stmt = store.conn.prepare(sql())?;
    let mut rows = stmt.query([i64::MAX])?;
    let mut summaries = Vec::new();
    let mut paths = Vec::new();
    while let Some(row) = rows.next()? {
        let fid: i32 = row.get(1)?;
        let fidelity = match fid {
            0 => "low",
            1 => "medium",
            2 => "high",
            3 => "edit",
            4 => "verbatim",
            _ => "low",
        };
        let file_path: String = row.get(0)?;
        let owned_fidelity = if projected {
            None
        } else {
            Some(fidelity.to_string())
        };
        let raw_tokens: i64 = row.get(2)?;
        let compressed_tokens: i64 = row.get(3)?;
        let (timestamp, timestamp_bytes) = if projected {
            let value: TextLength = row.get(4)?;
            (None, value.0)
        } else {
            let value: String = row.get(4)?;
            let len = value.len();
            (Some(value), len)
        };
        let delta_count: i64 = row.get(5)?;
        if observe {
            work.rows += 1;
            work.conversions += 6;
            work.paths += 1;
            work.owned_strings += if projected { 1 } else { 3 };
            work.owned_bytes += file_path.len()
                + if projected {
                    0
                } else {
                    fidelity.len() + timestamp_bytes
                };
            work.validated_timestamp_bytes += timestamp_bytes;
            work.count_results += 1;
            work.delta_total += delta_count;
        }
        if projected {
            paths.push(file_path);
        } else {
            if observe {
                work.summaries += 1;
            }
            summaries.push(PersistedContextSummary {
                file_path,
                fidelity: owned_fidelity.unwrap(),
                raw_tokens,
                compressed_tokens,
                updated_at: timestamp.unwrap(),
                delta_count,
            });
        }
    }
    drop(rows);
    if observe {
        work.capacity = if projected {
            paths.capacity()
        } else {
            summaries.capacity()
        };
        work.vm_steps = stmt.get_status(rusqlite::StatementStatus::VmStep);
        work.fullscan_steps = stmt.get_status(rusqlite::StatementStatus::FullscanStep);
    }
    Ok((
        if projected {
            Output::Paths(paths)
        } else {
            Output::Summaries(summaries)
        },
        work,
    ))
}
