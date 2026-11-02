//! Test-only shared neutral inspector, not a guard-kind producer.
pub(super) use super::typescript_route_guard_field_support::{
    Field, call, capture, retained_field,
};
use crate::ir::typescript_resolution::TypeScriptAuthority;
pub(super) const FIELDS: [&str; 4] = ["canMatch", "canDeactivate", "canLoad", "canActivateChild"];
pub(super) fn setup(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = super::typescript_route_guard_field_support::setup(form);
    for file in ["one.ts", "two.ts"] {
        let original = std::fs::read_to_string(root.path().join(file)).unwrap();
        let first = original
            .split_once("canActivate:[")
            .unwrap()
            .1
            .split(',')
            .next()
            .unwrap();
        let mut output = String::new();
        for (i, part) in original.split("canActivate:").enumerate() {
            if i == 0 {
                output.push_str(part);
                continue;
            }
            let end = part.find(']').unwrap() + 1;
            output.push_str(
                &FIELDS
                    .iter()
                    .map(|field| format!("{field}:{}", &part[..end]))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            output.push_str(&part[end..]);
        }
        output = output.replace(
            "path:'controls',component:",
            &format!("path:'controls',canActivate:[{first}],component:"),
        );
        std::fs::write(root.path().join(file), output).unwrap();
    }
    (root, config)
}
pub(super) fn all(a: &TypeScriptAuthority) -> Vec<(&'static str, Vec<Field>)> {
    FIELDS
        .into_iter()
        .map(|field| (field, retained_field(a, field)))
        .collect()
}
