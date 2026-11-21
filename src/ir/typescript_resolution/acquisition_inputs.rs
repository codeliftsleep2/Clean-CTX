//! Internal consumed-input receipts; existing persisted Witness truth is unchanged.
use super::Witness;
use std::{path::PathBuf, sync::Arc};

pub(super) type Inputs = Arc<[Input]>;
#[derive(Clone, Debug)]
pub(super) enum Input {
    Unaccounted,
    Observation(Arc<Witness>),
    Source(Arc<SourceInput>),
    Dependencies(Inputs),
}
#[derive(Clone, Debug)]
pub(super) struct SourceInput {
    pub path: PathBuf,
    pub bytes: Arc<str>,
    pub supplied: bool,
    pub accounted: bool,
    pub backing: Inputs,
}
#[derive(Default)]
pub(super) struct InputLog {
    values: Vec<Witness>,
    scopes: Vec<Vec<Input>>,
    roots: Inputs,
    enabled: bool,
}
impl std::ops::Deref for InputLog {
    type Target = [Witness];
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl InputLog {
    pub fn new(values: Vec<Witness>) -> Self {
        let roots = values
            .iter()
            .cloned()
            .map(|w| Input::Observation(Arc::new(w)))
            .collect::<Vec<_>>()
            .into();
        Self {
            values,
            scopes: Vec::new(),
            roots,
            enabled: false,
        }
    }
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    pub fn roots(&mut self) {
        self.attribute(&Arc::clone(&self.roots));
    }
    pub fn into_vec(self) -> Vec<Witness> {
        self.values
    }
    pub fn push(&mut self, witness: Witness) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(Input::Observation(Arc::new(witness.clone())));
        }
        self.values.push(witness);
    }
    pub fn begin(&mut self) -> usize {
        if !self.enabled {
            return usize::MAX;
        }
        let depth = self.scopes.len();
        self.scopes.push(Vec::new());
        depth
    }
    pub fn finish(&mut self, depth: usize) -> Inputs {
        if depth == usize::MAX {
            return Arc::from([]);
        }
        assert_eq!(self.scopes.len(), depth + 1, "native input scope nesting");
        let inputs: Inputs = self.scopes.pop().expect("input scope").into();
        self.attribute(&inputs);
        inputs
    }
    pub fn attribute(&mut self, inputs: &Inputs) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(if inputs.is_empty() {
                Input::Unaccounted
            } else {
                Input::Dependencies(Arc::clone(inputs))
            });
        }
    }
    pub fn source(&mut self, source: &Arc<SourceInput>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(Input::Source(Arc::clone(source)));
        }
    }
    pub fn import(&mut self, witnesses: Vec<Witness>, inputs: &Inputs) {
        // Identical constraints have the same currentness predicate. Keep the
        // first occurrence without collapsing distinct identities or receipts.
        let mut known: std::collections::HashSet<_> = self.values.iter().cloned().collect();
        for witness in witnesses {
            if known.insert(witness.clone()) {
                self.values.push(witness);
            }
        }
        self.attribute(inputs);
    }
}
pub(super) fn source_receipt(source: &Arc<SourceInput>) -> Inputs {
    vec![
        Input::Dependencies(Arc::clone(&source.backing)),
        Input::Source(Arc::clone(source)),
    ]
    .into()
}
/// Visit each recorded dependency node once; never enumerate or parse source files.
pub(super) fn visit(inputs: &Inputs, mut consume: impl FnMut(&Input)) {
    let mut pending = vec![(Arc::clone(inputs), 0)];
    let mut seen = std::collections::BTreeSet::new();
    while let Some((inputs, index)) = pending.pop() {
        if index == 0 && !seen.insert(Arc::as_ptr(&inputs) as *const Input as usize) {
            continue;
        }
        let Some(input) = inputs.get(index).cloned() else {
            continue;
        };
        pending.push((inputs, index + 1));
        if let Input::Dependencies(child) = input {
            pending.push((child, 0));
        } else {
            consume(&input);
        }
    }
}
