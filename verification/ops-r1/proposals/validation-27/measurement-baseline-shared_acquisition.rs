//! Scoped operation-local acquisition proofs; owner results remain private.
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Default)]
pub(crate) struct Operation {
    groups: Mutex<BTreeMap<String, Slot>>,
    #[cfg(test)]
    qualifications: AtomicUsize,
    #[cfg(test)]
    seal_checks: AtomicUsize,
    #[cfg(test)]
    qualified: AtomicUsize,
    #[cfg(test)]
    validation_checks: AtomicUsize,
    #[cfg(test)]
    qualification_inputs: AtomicUsize,
    #[cfg(test)]
    attempts: AtomicUsize,
    #[cfg(test)]
    completed: AtomicUsize,
    #[cfg(test)]
    visits: AtomicUsize,
}
impl Operation {
    pub(super) fn attempt(&self) {
        #[cfg(test)]
        self.attempts.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn completed(&self) {
        #[cfg(test)]
        self.completed.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn visit(&self) {
        #[cfg(test)]
        self.visits.fetch_add(1, Ordering::Relaxed);
    }
    #[cfg(test)]
    pub(crate) fn counts(&self) -> [usize; 3] {
        [
            self.attempts.load(Ordering::Relaxed),
            self.completed.load(Ordering::Relaxed),
            self.visits.load(Ordering::Relaxed),
        ]
    }
    #[cfg(test)]
    pub(crate) fn work_counts(&self) -> [usize; 5] {
        [
            self.qualifications.load(Ordering::Relaxed),
            self.qualified.load(Ordering::Relaxed),
            self.seal_checks.load(Ordering::Relaxed),
            self.validation_checks.load(Ordering::Relaxed),
            self.qualification_inputs.load(Ordering::Relaxed),
        ]
    }
}
#[cfg(test)]
impl Drop for Operation {
    fn drop(&mut self) {
        if std::env::var("OPS_R1_CAPTURE").ok().as_deref() == Some("1") {
            eprintln!(
                "OPS_R1_OPERATION acquisition={:?} qualification={:?}",
                self.counts(),
                self.work_counts()
            );
        }
    }
}
pub(crate) fn operation() -> Arc<Operation> {
    let operation = Arc::new(Operation::default());
    #[cfg(all(test, feature = "angular"))]
    OBSERVED.with(|slot| {
        if let Some(operations) = slot.borrow_mut().as_mut() {
            operations.push(Arc::clone(&operation));
        }
    });
    operation
}
#[cfg(all(test, feature = "angular"))]
thread_local! {
    static OBSERVED: std::cell::RefCell<Option<Vec<Arc<Operation>>>> = const { std::cell::RefCell::new(None) };
}
#[cfg(all(test, feature = "angular"))]
pub(crate) fn observe<T>(run: impl FnOnce() -> T) -> (T, Vec<Arc<Operation>>) {
    struct Restore(Option<Vec<Arc<Operation>>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            OBSERVED.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let saved = OBSERVED.with(|slot| slot.replace(Some(Vec::new())));
    let _restore = Restore(saved);
    let value = run();
    let operations = OBSERVED.with(|slot| slot.borrow_mut().take().expect("operation observation"));
    (value, operations)
}

use super::acquisition_inputs::{Input, Inputs, SourceInput, visit};
use super::{Resolver, Witness, project::Project, syntax::Unit};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Mutex,
};
pub(super) type ProgramUnits = Arc<BTreeMap<PathBuf, Unit>>;
type Slot = Arc<Mutex<Option<Arc<ProjectProof>>>>;
pub(super) struct ProjectProof {
    pub units: ProgramUnits,
    pub inputs: Inputs,
    pub sources: Arc<BTreeMap<PathBuf, Arc<SourceInput>>>,
    pub supplied_sources: Vec<Arc<SourceInput>>,
    pub program: super::program::ProgramEvidence,
    pub libraries: Vec<super::compiler_libraries::LibraryRequest>,
    pub types: Vec<super::automatic_types::AutomaticTypeRequest>,
    pub directives: Vec<super::program_directives::DirectiveRequest>,
    pub entries: Vec<super::package_evidence::ConsumedModuleEntry>,
}
#[derive(Clone)]
pub(super) struct BoundProgram {
    pub proof: Arc<ProjectProof>,
    pub inputs: Inputs,
    pub sources: Arc<BTreeMap<PathBuf, Arc<SourceInput>>>,
}
pub(super) struct Qualified {
    pub bound: BoundProgram,
    pub witnesses: Vec<Witness>,
}
impl Operation {
    pub(super) fn slot(&self, basis: String) -> Slot {
        let mut groups = self
            .groups
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Arc::clone(
            groups
                .entry(basis)
                .or_insert_with(|| Arc::new(Mutex::new(None))),
        )
    }
    #[cfg(test)]
    pub(crate) fn qualification_counts(&self) -> [usize; 3] {
        [
            self.qualifications.load(Ordering::Relaxed),
            self.qualified.load(Ordering::Relaxed),
            self.validation_checks.load(Ordering::Relaxed),
        ]
    }
}
/// Preserve directory-dependent lookup unless explicit types disable that lookup.
pub(super) fn basis(resolver: &Resolver, file: &Path, project: &Project) -> Option<String> {
    let config = project.config.as_ref()?;
    // Aliased request sources stay independent until route-equivalence is established.
    if !resolver.host.supplied_inputs.contains_key(file) {
        return None;
    }
    // Automatic type discovery starts at the request directory. An explicit
    // empty list performs no such lookup; libraries use the selected config,
    // while module and directive resolution use each acquired unit's own path.
    let directory = if project
        .options
        .get("types")
        .and_then(|option| option.value.as_array())
        .is_some_and(Vec::is_empty)
    {
        None
    } else {
        file.parent()
    };
    serde_json::to_string(&serde_json::json!({
        "config": config, "root": project.root, "base": project.base,
        "mode": project.mode, "paths": project.paths, "conditions": project.conditions,
        "options": project.options, "ownership": project.ownership,
        "compiler": project.compiler_version, "extensions": format!("{:?}", project.extensions),
        "directory": directory, "extension": file.extension().and_then(|s| s.to_str()),
        "roots": resolver.host.roots, "aliases": resolver.host.aliases,
        "demands": resolver.acquisition_policy,
    }))
    .ok()
}
pub(super) struct AcquiredSources {
    pub all: Arc<BTreeMap<PathBuf, Arc<SourceInput>>>,
    pub supplied: Vec<Arc<SourceInput>>,
}
pub(super) fn reusable_sources(inputs: &Inputs, _operation: &Operation) -> Option<AcquiredSources> {
    if inputs.is_empty() {
        return None;
    }
    let mut sources = BTreeMap::new();
    let mut supplied = Vec::new();
    let mut valid = true;
    visit(inputs, |input| match input {
        Input::Observation(witness) => {
            #[cfg(test)]
            _operation.seal_checks.fetch_add(1, Ordering::Relaxed);
            valid &= witness.current();
        }
        Input::Source(source) => {
            valid &= source.accounted;
            if let Some(previous) = sources.insert(source.path.clone(), Arc::clone(source)) {
                valid &= previous.bytes == source.bytes && previous.supplied == source.supplied;
            } else if source.supplied {
                supplied.push(Arc::clone(source));
            }
        }
        Input::Unaccounted => valid = false,
        Input::Dependencies(_) => unreachable!("visitor expands dependencies"),
    });
    valid.then(|| AcquiredSources {
        all: Arc::new(sources),
        supplied,
    })
}
impl ProjectProof {
    pub(super) fn qualify(
        self: &Arc<Self>,
        resolver: &mut Resolver,
        _operation: &Operation,
    ) -> Option<Qualified> {
        #[cfg(test)]
        _operation.qualifications.fetch_add(1, Ordering::Relaxed);
        if !resolver.can_use_program(self) {
            return None;
        }
        let mut sources = BTreeMap::new();
        // Check only the recipient's already consumed inputs, not a second unit traversal.
        for (path, cached) in &resolver.host.sources {
            if let Some(source) = self.sources.get(path)
                && cached.as_deref() != Some(source.bytes.as_ref())
            {
                return None;
            }
            if self.sources.contains_key(path)
                && resolver
                    .host
                    .source_inputs
                    .get(path)
                    .is_some_and(|input| !input.accounted)
            {
                return None;
            }
        }
        for (path, supplied) in &resolver.host.supplied_inputs {
            let source = self.sources.get(path)?;
            if supplied.as_ref() != source.bytes.as_ref() {
                return None;
            }
            sources.insert(
                path.clone(),
                Arc::clone(resolver.host.source_inputs.get(path)?),
            );
        }
        for source in &self.supplied_sources {
            if sources.contains_key(&source.path) {
                continue;
            }
            // Another owner's supplied bytes cannot authorize our dependency.
            let disk = resolver.host.read(&source.path)?;
            if disk.as_ref() != source.bytes.as_ref() {
                return None;
            }
            let input = resolver.host.source_inputs.get(&source.path)?;
            if input.supplied {
                return None;
            }
            sources.insert(source.path.clone(), Arc::clone(input));
        }
        let mut inputs = Vec::new();
        let mut witnesses = Vec::new();
        let mut valid = true;
        let mut consume = |input: &Input| {
            #[cfg(test)]
            _operation
                .qualification_inputs
                .fetch_add(1, Ordering::Relaxed);
            if let Input::Observation(witness) = input {
                if matches!(witness.as_ref(), Witness::Content { path, .. } if resolver.host.supplied_inputs.contains_key(path))
                {
                    return;
                }
                #[cfg(test)]
                _operation.validation_checks.fetch_add(1, Ordering::Relaxed);
                valid &= witness.current();
                witnesses.push(witness.as_ref().clone());
                inputs.push(Input::Observation(Arc::clone(witness)));
            } else if let Input::Source(source) = input {
                inputs.push(Input::Source(
                    sources
                        .get(&source.path)
                        .cloned()
                        .unwrap_or_else(|| Arc::clone(source)),
                ));
            } else if matches!(input, Input::Unaccounted) {
                valid = false;
            }
        };
        visit(&self.inputs, &mut consume);
        // Role-bound source backing may add the necessary disk proof for a leader seed.
        for source in sources.values() {
            visit(&source.backing, &mut consume);
        }
        if !valid {
            return None;
        }
        #[cfg(test)]
        _operation.qualified.fetch_add(1, Ordering::Relaxed);
        Some(Qualified {
            bound: BoundProgram {
                proof: Arc::clone(self),
                inputs: inputs.into(),
                sources: Arc::new(sources),
            },
            witnesses,
        })
    }
}
