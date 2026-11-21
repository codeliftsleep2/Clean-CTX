use super::artifact::SemanticCompilationArtifact;
use super::compile::{CompilationRequest, compile_source_artifact_in_operation};
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) struct CompilationJob {
    pub(crate) authorized_roots: Vec<std::path::PathBuf>,
    pub(crate) typescript_project: Option<std::path::PathBuf>,
    pub(crate) file_path: String,
    pub(crate) source: Arc<String>,
    pub(crate) fidelity: crate::compression::Fidelity,
    pub(crate) path_alias: String,
    pub(crate) previous_version: u64,
    pub(crate) canonical_owner: String,
    pub(crate) skip_set: Option<HashSet<String>>,
}

pub(crate) struct CompilationBatchResult {
    pub(crate) file_path: String,
    pub(crate) result: Result<SemanticCompilationArtifact, crate::error::CleanCtxError>,
}

pub(crate) struct SemanticCompilerExecutor {
    pool: rayon::ThreadPool,
    #[cfg(test)]
    full_semantic_parses: std::sync::atomic::AtomicUsize,
}

impl SemanticCompilerExecutor {
    pub(crate) fn new(worker_count: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(worker_count.max(1))
            .thread_name(|index| format!("clean-ctx-compiler-{index}"))
            .build()
            .expect("bounded semantic compiler pool must initialize");
        Self {
            pool,
            #[cfg(test)]
            full_semantic_parses: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub(crate) fn default_worker_count() -> usize {
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .saturating_sub(1)
            .max(1)
    }

    pub(crate) fn compile(
        &self,
        config: &crate::config::CleanCtxConfig,
        jobs: Vec<CompilationJob>,
    ) -> Vec<CompilationBatchResult> {
        #[cfg(test)]
        self.full_semantic_parses
            .fetch_add(jobs.len(), std::sync::atomic::Ordering::SeqCst);
        #[cfg(feature = "typescript")]
        let operation = crate::ir::typescript_resolution::shared_acquisition::operation();
        self.pool.install(|| {
            let results: Vec<_> = jobs
                .into_par_iter()
                .map(|job| {
                    #[cfg(all(test, feature = "angular"))]
                    crate::ops_r1_observation::record(4, 1);
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        compile_source_artifact_in_operation(
                            CompilationRequest {
                                file_path: &job.file_path,
                                source: &job.source,
                                fidelity: job.fidelity,
                                config,
                                authorized_roots: &job.authorized_roots,
                                typescript_project: job.typescript_project.as_deref(),
                                focus: None,
                                skip_set: job.skip_set.as_ref(),
                                path_alias: &job.path_alias,
                                previous_version: job.previous_version,
                                canonical_owner: &job.canonical_owner,
                                #[cfg(feature = "csharp")]
                                csharp_authority: None,
                            },
                            #[cfg(feature = "typescript")]
                            Some(std::sync::Arc::clone(&operation)),
                        )
                    }))
                    .unwrap_or_else(|_| {
                        Err(crate::error::CleanCtxError::Ir(
                            "semantic compilation worker panicked".to_string(),
                        ))
                    });
                    #[cfg(all(test, feature = "angular"))]
                    crate::ops_r1_observation::record(5, 1);
                    CompilationBatchResult {
                        file_path: job.file_path,
                        result,
                    }
                })
                .collect();
            #[cfg(all(test, feature = "angular"))]
            crate::ops_r1_observation::batch_results(results.len());
            results
        })
    }
}

impl crate::mcp::McpState {
    pub(crate) fn semantic_compiler(&self) -> &SemanticCompilerExecutor {
        &self.semantic_compiler
    }

    #[cfg(test)]
    #[cfg(any(feature = "csharp", feature = "angular"))]
    pub(crate) fn reset_test_full_semantic_parses(&self) {
        self.semantic_compiler
            .full_semantic_parses
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    #[cfg(any(feature = "csharp", feature = "angular"))]
    pub(crate) fn test_full_semantic_parses(&self) -> usize {
        self.semantic_compiler
            .full_semantic_parses
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}
