use serde::{Deserialize, Serialize};

pub const KINDS: &[&str] = &[
    "MISSION",
    "IMPLEMENTATION",
    "EXPERIMENT",
    "RESEARCH",
    "AUTHORITY",
    "INVARIANT",
    "DEBT",
];
pub const STATUSES: &[&str] = &[
    "PROPOSED",
    "INVESTIGATING",
    "FROZEN",
    "IMPLEMENTATION_READY",
    "IMPLEMENTED",
    "VALIDATED",
    "RECLOSED",
    "BANKED",
    "BLOCKED",
    "FALSIFIED",
    "SUPERSEDED",
    "DEBT",
];
pub const RELATIONS: &[&str] = &[
    "DEPENDS_ON",
    "IMPLEMENTS",
    "VALIDATES",
    "RECLOSES",
    "BANKS",
    "BLOCKED_BY",
    "SUPERSEDES",
    "PRESERVES",
    "FALSIFIES",
    "USES_EVIDENCE",
    "DEEPENS",
    "PRODUCES_AUTHORITY",
    "CONSUMES_AUTHORITY",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    #[serde(rename = "node")]
    pub id: String,
    pub kind: String,
    pub status: String,
    pub title: String,
    pub track: String,
    pub artifact: Option<String>,
    pub verdict: Option<String>,
    pub commit: Option<String>,
    pub note: Option<String>,
    pub relationships: Vec<Relationship>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub kind: String,
    pub target: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub path: String,
    pub node: Option<String>,
    pub message: String,
}
impl Diagnostic {
    pub fn new(code: &str, path: &str, node: Option<&str>, message: &str) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            node: node.map(str::to_owned),
            message: message.into(),
        }
    }
}
pub struct Candidate {
    pub path: String,
    pub source_sha: String,
    pub bytes: Vec<u8>,
    pub declaration: Declaration,
    pub revision: String,
    pub evidence: Vec<serde_json::Value>,
}
