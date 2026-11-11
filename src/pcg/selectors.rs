//! Exact three-valued conjunction/union. No lexical discovery or implicit time.
use super::{Error, Result, identity, model::Selector, require, security};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub kind: String,
    pub file: String,
    pub source_sha256: String,
    pub start: u64,
    pub end: u64,
    pub name: String,
    pub declaration_kind: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub workspace: String,
    pub file: Option<String>,
    pub entity: Option<Declaration>,
    pub operation: Option<String>,
    pub task: Option<String>,
    pub at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Truth {
    False,
    Unknown,
    True,
}
impl Truth {
    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            _ => Self::False,
        }
    }
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            _ => Self::True,
        }
    }
}

pub(super) fn instant(value: &str) -> Result<()> {
    require(value.len() == 20 && value.ends_with('Z'), "invalid_time")?;
    let time = chrono::DateTime::parse_from_rfc3339(value).map_err(|_| Error("invalid_time"))?;
    require(
        time.format("%Y-%m-%dT%H:%M:%SZ").to_string() == value && !value.contains(":60"),
        "invalid_time",
    )
}

impl Subject {
    pub(super) fn validate(&self) -> Result<()> {
        identity::id(&self.workspace)?;
        instant(&self.at)?;
        if let Some(file) = &self.file {
            security::relative(file)?;
        }
        for value in [&self.operation, &self.task].into_iter().flatten() {
            identity::id(value)?;
        }
        if let Some(entity) = &self.entity {
            require(
                entity.kind == "declaration"
                    && entity.end > entity.start
                    && entity.end <= i64::MAX as u64
                    && !entity.name.is_empty()
                    && !entity.declaration_kind.is_empty(),
                "invalid_entity",
            )?;
            security::relative(&entity.file)?;
            identity::hex(&entity.source_sha256, 64)?;
            require(
                self.file.as_ref().is_none_or(|f| f == &entity.file),
                "invalid_entity",
            )?;
        }
        Ok(())
    }
    pub(super) fn file(&self) -> Option<&str> {
        self.file
            .as_deref()
            .or_else(|| self.entity.as_ref().map(|e| e.file.as_str()))
    }
}

fn dimension(selector: Option<&str>, requested: Option<&str>) -> Truth {
    match (selector, requested) {
        (None, _) => Truth::True,
        (Some(_), None) => Truth::Unknown,
        (Some(a), Some(b)) if a == b => Truth::True,
        _ => Truth::False,
    }
}

pub(super) fn evaluate(selector: &Selector, subject: &Subject) -> Truth {
    let mut result = dimension(Some(&selector.workspace), Some(&subject.workspace));
    let file = selector
        .file
        .as_deref()
        .or_else(|| selector.entity.as_ref().map(|e| e.file.as_str()));
    result = result.and(dimension(file, subject.file()));
    result = result.and(dimension(
        selector.operation.as_deref(),
        subject.operation.as_deref(),
    ));
    result = result.and(dimension(selector.task.as_deref(), subject.task.as_deref()));
    if selector
        .from
        .as_ref()
        .is_some_and(|from| subject.at < *from)
        || selector
            .until
            .as_ref()
            .is_some_and(|until| subject.at >= *until)
    {
        result = Truth::False;
    }
    if let Some(entity) = &selector.entity {
        result = result.and(match &subject.entity {
            None => Truth::Unknown,
            Some(requested) => {
                if serde_json::to_value(entity).ok() == serde_json::to_value(requested).ok() {
                    Truth::True
                } else {
                    Truth::False
                }
            }
        });
    }
    result
}
