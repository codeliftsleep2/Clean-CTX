//! Private, source-bound invocation occurrence evidence.
//!
//! This side channel is intentionally outside CompiledIR, canonical encodings,
//! persistence, semantic edges, and public MCP schemas. Phase 1 establishes the
//! lifecycle and validation boundary only; language adapters populate it later.

use crate::cache::content_hash;

/// A zero-based, half-open UTF-8 byte range: `[start_byte, end_byte)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceSpan {
    start_byte: usize,
    end_byte: usize,
}

impl SourceSpan {
    pub(crate) fn new(start_byte: usize, end_byte: usize) -> Result<Self, EvidenceSliceError> {
        if start_byte > end_byte {
            return Err(EvidenceSliceError::Reversed {
                start_byte,
                end_byte,
            });
        }
        Ok(Self {
            start_byte,
            end_byte,
        })
    }

    pub(crate) fn start_byte(self) -> usize {
        self.start_byte
    }

    pub(crate) fn end_byte(self) -> usize {
        self.end_byte
    }

    fn contains(self, inner: Self) -> bool {
        self.start_byte <= inner.start_byte && inner.end_byte <= self.end_byte
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArgumentQualifier {
    Plain,
    Spread,
    Named { name: SourceSpan },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArgumentOccurrence {
    span: SourceSpan,
    qualifier: ArgumentQualifier,
}

impl ArgumentOccurrence {
    pub(crate) fn new(span: SourceSpan, qualifier: ArgumentQualifier) -> Self {
        Self { span, qualifier }
    }

    pub(crate) fn span(&self) -> SourceSpan {
        self.span
    }

    pub(crate) fn qualifier(&self) -> &ArgumentQualifier {
        &self.qualifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WrittenCallee {
    Free {
        terminal: SourceSpan,
    },
    Member {
        receiver: SourceSpan,
        terminal: SourceSpan,
    },
    Unsupported {
        function: Option<SourceSpan>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclarationContextKind {
    Callable,
    VariableInitializer,
    FieldInitializer,
    PropertyInitializer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclarationContext {
    kind: DeclarationContextKind,
    declaration: SourceSpan,
    written_name: Option<SourceSpan>,
}

impl DeclarationContext {
    pub(crate) fn new(
        kind: DeclarationContextKind,
        declaration: SourceSpan,
        written_name: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind,
            declaration,
            written_name,
        }
    }

    pub(crate) fn kind(&self) -> DeclarationContextKind {
        self.kind
    }

    pub(crate) fn declaration(&self) -> SourceSpan {
        self.declaration
    }

    pub(crate) fn written_name(&self) -> Option<SourceSpan> {
        self.written_name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentSite {
    Receiver,
    Argument { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvocationParent {
    invocation: SourceSpan,
    site: ParentSite,
}

impl InvocationParent {
    pub(crate) fn new(invocation: SourceSpan, site: ParentSite) -> Self {
        Self { invocation, site }
    }

    pub(crate) fn invocation(&self) -> SourceSpan {
        self.invocation
    }

    pub(crate) fn site(&self) -> ParentSite {
        self.site
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvocationOccurrence {
    span: SourceSpan,
    callee: WrittenCallee,
    arguments: Vec<ArgumentOccurrence>,
    type_arguments: Vec<SourceSpan>,
    declaration_context: Option<DeclarationContext>,
    parent: Option<InvocationParent>,
}

impl InvocationOccurrence {
    pub(crate) fn new(
        span: SourceSpan,
        callee: WrittenCallee,
        arguments: Vec<ArgumentOccurrence>,
        type_arguments: Vec<SourceSpan>,
        declaration_context: Option<DeclarationContext>,
        parent: Option<InvocationParent>,
    ) -> Self {
        Self {
            span,
            callee,
            arguments,
            type_arguments,
            declaration_context,
            parent,
        }
    }

    pub(crate) fn span(&self) -> SourceSpan {
        self.span
    }

    pub(crate) fn callee(&self) -> &WrittenCallee {
        &self.callee
    }

    pub(crate) fn arguments(&self) -> &[ArgumentOccurrence] {
        &self.arguments
    }

    pub(crate) fn type_arguments(&self) -> &[SourceSpan] {
        &self.type_arguments
    }

    pub(crate) fn declaration_context(&self) -> Option<&DeclarationContext> {
        self.declaration_context.as_ref()
    }

    pub(crate) fn parent(&self) -> Option<&InvocationParent> {
        self.parent.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvocationEvidenceSet {
    source_id: String,
    source_hash: String,
    occurrences: Vec<InvocationOccurrence>,
}

impl InvocationEvidenceSet {
    pub(crate) fn new(
        source_id: String,
        source: &str,
        mut occurrences: Vec<InvocationOccurrence>,
    ) -> Self {
        occurrences.sort_by(|left, right| {
            left.span
                .start_byte
                .cmp(&right.span.start_byte)
                .then_with(|| right.span.end_byte.cmp(&left.span.end_byte))
        });
        Self {
            source_id,
            source_hash: content_hash(source.as_bytes()),
            occurrences,
        }
    }

    pub(crate) fn source_id(&self) -> &str {
        &self.source_id
    }

    pub(crate) fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub(crate) fn view<'a>(
        &'a self,
        expected_source_id: &str,
        source: &'a str,
    ) -> Result<InvocationEvidenceView<'a>, EvidenceMismatch> {
        if expected_source_id != self.source_id {
            return Err(EvidenceMismatch::SourceIdentityMismatch);
        }
        if content_hash(source.as_bytes()) != self.source_hash {
            return Err(EvidenceMismatch::SourceHashMismatch);
        }
        Ok(InvocationEvidenceView {
            evidence: self,
            source,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenceMismatch {
    SourceIdentityMismatch,
    SourceHashMismatch,
}

impl std::fmt::Display for EvidenceMismatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceIdentityMismatch => formatter.write_str("invocation evidence source identity mismatch"),
            Self::SourceHashMismatch => formatter.write_str("invocation evidence source hash mismatch"),
        }
    }
}

impl std::error::Error for EvidenceMismatch {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenceSliceError {
    Reversed { start_byte: usize, end_byte: usize },
    OutOfBounds { span: SourceSpan, source_len: usize },
    NotUtf8Boundary { span: SourceSpan },
}

impl std::fmt::Display for EvidenceSliceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reversed {
                start_byte,
                end_byte,
            } => write!(formatter, "reversed evidence span [{start_byte}, {end_byte})"),
            Self::OutOfBounds { span, source_len } => write!(
                formatter,
                "evidence span [{}, {}) exceeds source length {source_len}",
                span.start_byte, span.end_byte
            ),
            Self::NotUtf8Boundary { span } => write!(
                formatter,
                "evidence span [{}, {}) is not on UTF-8 boundaries",
                span.start_byte, span.end_byte
            ),
        }
    }
}

impl std::error::Error for EvidenceSliceError {}

pub(crate) struct InvocationEvidenceView<'a> {
    evidence: &'a InvocationEvidenceSet,
    source: &'a str,
}

impl<'a> InvocationEvidenceView<'a> {
    pub(crate) fn occurrences(&self) -> &[InvocationOccurrence] {
        &self.evidence.occurrences
    }

    pub(crate) fn slice(&self, span: SourceSpan) -> Result<&'a str, EvidenceSliceError> {
        if !SourceSpan::new(0, self.source.len())
            .expect("zero-to-length span is valid")
            .contains(span)
        {
            return Err(EvidenceSliceError::OutOfBounds {
                span,
                source_len: self.source.len(),
            });
        }
        if !self.source.is_char_boundary(span.start_byte)
            || !self.source.is_char_boundary(span.end_byte)
        {
            return Err(EvidenceSliceError::NotUtf8Boundary { span });
        }
        Ok(&self.source[span.start_byte..span.end_byte])
    }
}

#[cfg(test)]
#[path = "../tests/ir/invocation_evidence.rs"]
mod tests;
