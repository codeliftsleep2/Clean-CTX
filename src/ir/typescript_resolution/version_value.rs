//! Compiler version precedence; build metadata is validated but does not order versions.
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Version {
    pub core: [u32; 3],
    pub prerelease: Vec<String>,
}

impl Version {
    pub(super) fn new(
        core: [u32; 3],
        prerelease: Option<&str>,
        build: Option<&str>,
    ) -> Result<Self, String> {
        if let Some(build) = build {
            identifiers(build, false)?;
        }
        Ok(Self {
            core,
            prerelease: prerelease
                .map(|value| identifiers(value, true))
                .transpose()?
                .unwrap_or_default(),
        })
    }
    pub(super) fn floor(mut self) -> Self {
        self.prerelease = vec!["0".into()];
        self
    }
}

fn identifiers(value: &str, prerelease: bool) -> Result<Vec<String>, String> {
    value
        .split('.')
        .map(|part| {
            if part.is_empty()
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return Err("Invalid compiler version identifier".into());
            }
            if prerelease {
                let numeric = part.bytes().all(|byte| byte.is_ascii_digit());
                if (numeric && part.len() > 1 && part.starts_with('0'))
                    || (!numeric
                        && !part.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '-'))
                {
                    return Err("Invalid compiler prerelease identifier".into());
                }
            }
            Ok(part.to_string())
        })
        .collect()
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let core = self.core.cmp(&other.core);
        if core != Ordering::Equal {
            return core;
        }
        if self.prerelease.is_empty() {
            return if other.prerelease.is_empty() {
                Ordering::Equal
            } else {
                Ordering::Greater
            };
        }
        if other.prerelease.is_empty() {
            return Ordering::Less;
        }
        for (left, right) in self.prerelease.iter().zip(&other.prerelease) {
            let numeric_left = left.bytes().all(|byte| byte.is_ascii_digit());
            let numeric_right = right.bytes().all(|byte| byte.is_ascii_digit());
            let order = match (numeric_left, numeric_right) {
                (true, true) => left.len().cmp(&right.len()).then_with(|| left.cmp(right)),
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => left.cmp(right),
            };
            if order != Ordering::Equal {
                return order;
            }
        }
        self.prerelease.len().cmp(&other.prerelease.len())
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
