use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct EnvironmentFacts {
    pub policy: &'static str,
    pub inherited_names: Vec<String>,
    pub stripped_names: Vec<String>,
    pub overridden_names: Vec<String>,
    pub rejected_non_unicode_names: usize,
    pub named_rustup_toolchain: Option<String>,
    pub rustup_toolchain_rejected_as_path: bool,
    pub credential_variables_stripped: usize,
    pub proxy_variables_stripped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoCheckEnvironment {
    entries: BTreeMap<OsString, OsString>,
    facts: EnvironmentFacts,
}

impl CargoCheckEnvironment {
    pub fn from_snapshot<I, K, V>(snapshot: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        let mut entries = BTreeMap::new();
        let mut facts = EnvironmentFacts {
            policy: "cargo-check-strict-v1",
            ..EnvironmentFacts::default()
        };

        for (name, value) in snapshot {
            let name = name.into();
            let value = value.into();
            let Some(name_text) = name.to_str() else {
                facts.rejected_non_unicode_names += 1;
                continue;
            };
            let normalized = name_text.to_ascii_uppercase();

            if normalized == "RUSTUP_TOOLCHAIN" {
                if is_named_toolchain(&value) {
                    facts.named_rustup_toolchain = value.to_str().map(str::to_owned);
                    inherit(&mut entries, &mut facts, name, value);
                } else {
                    facts.rustup_toolchain_rejected_as_path = true;
                    facts.stripped_names.push(normalized);
                }
            } else if is_credential_name(&normalized) {
                facts.credential_variables_stripped += 1;
                facts.stripped_names.push(normalized);
            } else if is_proxy_name(&normalized) {
                facts.proxy_variables_stripped += 1;
                facts.stripped_names.push(normalized);
            } else if is_inherited_runtime_name(&normalized) {
                inherit(&mut entries, &mut facts, name, value);
            } else {
                facts.stripped_names.push(normalized);
            }
        }

        override_value(&mut entries, &mut facts, "RUSTUP_AUTO_INSTALL", "0");
        override_value(&mut entries, &mut facts, "RUSTC_WRAPPER", "");
        override_value(&mut entries, &mut facts, "RUSTC_WORKSPACE_WRAPPER", "");
        override_value(&mut entries, &mut facts, "CARGO_TERM_COLOR", "never");
        override_value(
            &mut entries,
            &mut facts,
            "RUSTUP_TERM_PROGRESS_WHEN",
            "never",
        );

        facts.inherited_names.sort();
        facts.inherited_names.dedup();
        facts.stripped_names.sort();
        facts.stripped_names.dedup();
        facts.overridden_names.sort();
        facts.overridden_names.dedup();
        Self { entries, facts }
    }

    pub fn entries(&self) -> &BTreeMap<OsString, OsString> {
        &self.entries
    }

    pub fn facts(&self) -> &EnvironmentFacts {
        &self.facts
    }
}

fn inherit(
    entries: &mut BTreeMap<OsString, OsString>,
    facts: &mut EnvironmentFacts,
    name: OsString,
    value: OsString,
) {
    facts
        .inherited_names
        .push(name.to_string_lossy().to_ascii_uppercase());
    entries.insert(name, value);
}

fn override_value(
    entries: &mut BTreeMap<OsString, OsString>,
    facts: &mut EnvironmentFacts,
    name: &'static str,
    value: &'static str,
) {
    remove_case_insensitive(entries, name);
    entries.insert(OsString::from(name), OsString::from(value));
    facts.overridden_names.push(name.to_owned());
}

fn remove_case_insensitive(entries: &mut BTreeMap<OsString, OsString>, target: &str) {
    entries.retain(|name, _| !name.to_string_lossy().eq_ignore_ascii_case(target));
}

fn is_named_toolchain(value: &OsStr) -> bool {
    let Some(value) = value.to_str() else {
        return false;
    };
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && !value.contains('/')
        && !value.contains('\\')
}

fn is_inherited_runtime_name(name: &str) -> bool {
    matches!(
        name,
        "PATH"
            | "HOME"
            | "USERPROFILE"
            | "HOMEDRIVE"
            | "HOMEPATH"
            | "CARGO_HOME"
            | "RUSTUP_HOME"
            | "SYSTEMROOT"
            | "WINDIR"
            | "COMSPEC"
            | "TMP"
            | "TEMP"
            | "TMPDIR"
            | "INCLUDE"
            | "LIB"
            | "LIBPATH"
            | "VCINSTALLDIR"
            | "VSINSTALLDIR"
            | "WINDOWSSDKDIR"
            | "WINDOWSSDKVERSION"
            | "UCRTVERSION"
            | "LANG"
    ) || name.starts_with("LC_")
}

fn is_proxy_name(name: &str) -> bool {
    matches!(
        name,
        "HTTP_PROXY" | "HTTPS_PROXY" | "ALL_PROXY" | "NO_PROXY"
    )
}

fn is_credential_name(name: &str) -> bool {
    [
        "TOKEN",
        "PASSWORD",
        "PASSWD",
        "SECRET",
        "API_KEY",
        "ACCESS_KEY",
        "PRIVATE_KEY",
        "CREDENTIAL",
    ]
    .iter()
    .any(|marker| name.contains(marker))
}
