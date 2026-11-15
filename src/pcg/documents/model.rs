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
    /// Source extension is projected separately; legacy metadata/hashes keep their shape.
    #[serde(skip)]
    pub knowledge: Option<super::content::Spec>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<DiagnosticLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admission_disposition: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_publication: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governing_authority: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_descriptor: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_assurance: Option<&'static str>,
}
#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticLocation {
    pub line: usize,
    pub column: usize,
    pub field: String,
}
impl Diagnostic {
    pub fn new(code: &str, path: &str, node: Option<&str>, message: &str) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            node: node.map(str::to_owned),
            message: message.into(),
            location: None,
            observed: None,
            admission_disposition: None,
            accepted_publication: None,
            governing_authority: None,
            change_descriptor: None,
            authorization_digest: None,
            authorization_assurance: None,
        }
    }

    pub fn at_field(mut self, line: usize, column: usize, field: &str, observed: &str) -> Self {
        self.location = Some(DiagnosticLocation {
            line,
            column,
            field: field.into(),
        });
        self.observed = Some(observed.into());
        self.admission_disposition = Some("rejected");
        self.accepted_publication = Some(false);
        self
    }
}
pub struct Candidate {
    pub path: String,
    pub source_sha: String,
    pub bytes: std::sync::Arc<Vec<u8>>,
    pub declaration: Declaration,
    pub revision: String,
    pub evidence: Vec<serde_json::Value>,
    pub knowledge: Option<super::content::Knowledge>,
}
