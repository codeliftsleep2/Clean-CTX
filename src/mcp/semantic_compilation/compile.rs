use crate::compression::Fidelity;
use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;
use std::collections::HashSet;
use std::path::Path;

#[cfg(test)]
pub(crate) static TEST_INJECTED_IR_FAILURE: std::sync::Mutex<Option<String>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
pub(crate) static TEST_INJECTED_SOURCE_FAILURE: std::sync::Mutex<Option<String>> =
    std::sync::Mutex::new(None);

pub(crate) struct CompilationRequest<'a> {
    pub(crate) file_path: &'a str,
    pub(crate) source: &'a str,
    pub(crate) fidelity: Fidelity,
    pub(crate) config: &'a crate::config::CleanCtxConfig,
    pub(crate) focus: Option<&'a HashSet<String>>,
    pub(crate) skip_set: Option<&'a HashSet<String>>,
    pub(crate) path_alias: &'a str,
    pub(crate) previous_version: u64,
    pub(crate) canonical_owner: &'a str,
    #[cfg(feature = "csharp")]
    pub(crate) csharp_authority: Option<&'a [crate::ir::csharp_exact_calls::Declaration]>,
}

pub(crate) fn compile_source_artifact(
    request: CompilationRequest<'_>,
) -> Result<SemanticCompilationArtifact, crate::error::CleanCtxError> {
    #[cfg(test)]
    crate::mcp::semantic_work_metrics::record_full_semantic_parse();
    #[cfg(test)]
    if let Ok(injected) = TEST_INJECTED_IR_FAILURE.lock()
        && let Some(reason) = injected.as_ref()
    {
        return Err(crate::error::CleanCtxError::Ir(format!(
            "injected IR failure: {reason}"
        )));
    }
    #[cfg(test)]
    if let Ok(injected) = TEST_INJECTED_SOURCE_FAILURE.lock()
        && let Some(marker) = injected.as_ref()
        && request.source.contains(marker)
    {
        return Err(crate::error::CleanCtxError::Ir(format!(
            "injected candidate IR failure: {marker}"
        )));
    }

    use crate::compression::language::language_for_extension;
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::csharp::CSharpLayer;
    use crate::ir::layers::java::JavaLayer;
    use crate::ir::layers::rust::RustLayer;
    use crate::ir::layers::typescript::TypeScriptLayer;

    let path = Path::new(request.file_path);
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("");
    let (language, query_string) = language_for_extension(extension).ok_or_else(|| {
        crate::error::CleanCtxError::Ir(format!("Unsupported file extension: .{extension}"))
    })?;
    let source_hash = crate::cache::content_hash(request.source.as_bytes());
    let mut compiler = IRCompiler::new();
    compiler.set_config(request.config.clone());
    #[cfg(feature = "csharp")]
    if let Some(authority) = request.csharp_authority {
        compiler.set_csharp_declaration_authority(authority.to_vec());
    }

    match extension {
        "ts" | "js" => compiler.add_language_layer(Box::new(TypeScriptLayer::new())),
        "cs" => compiler.add_language_layer(Box::new(CSharpLayer::new())),
        "rs" => compiler.add_language_layer(Box::new(RustLayer::new())),
        "java" => compiler.add_language_layer(Box::new(JavaLayer::new())),
        _ => {}
    }
    compiler.add_pattern_recognizer(Box::new(
        crate::ir::layers::patterns::CodePatternRecognizer::new(),
    ));
    compiler.add_pattern_recognizer(Box::new(
        crate::ir::patterns::CompressingPatternRecognizer::new(),
    ));
    let mut compiled = compiler.compile_focused(
        request.source,
        request.path_alias,
        Some(request.canonical_owner),
        language,
        query_string,
        request.fidelity,
        request.skip_set,
        request.focus,
    )?;
    crate::ir::type_aliases::apply_type_aliases_to_ir(
        &mut compiled.instructions,
        &request.config.type_aliases,
    );
    compiled.version = request.previous_version.saturating_add(1);
    SemanticCompilationArtifact::from_compiler(
        compiled,
        &mut compiler,
        source_hash,
        request.canonical_owner.to_string(),
        request.source,
        request.fidelity,
        path,
        request.config,
        extension,
    )
}
