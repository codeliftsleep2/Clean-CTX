//! Issues #153/#154: resolve declaration identity before rendering order.

use super::*;
use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::csharp_resolution::{CSharpResolvedClass, resolve_compilation_units};
use std::collections::BTreeSet;

fn compile(source: &str) -> IRCompiler {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
    compiler
        .compile_focused(
            source,
            "compilation-identity",
            Some("C:/repo/Identity.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    compiler
}

fn resolved(compiler: &IRCompiler) -> Vec<CSharpResolvedClass> {
    resolve_compilation_units(
        &[compiler
            .csharp_compilation_unit
            .clone()
            .expect("retained unit")],
        &known_csharp_type_identities(),
    )
}

#[test]
fn compilation_identity_constructor_cannot_shift_controller_actions() {
    let methods = [
        "[HttpGet] public void ActionA() {}",
        "[HttpPost] public void ActionB() {}",
        "[HttpPut] public void ActionC() {}",
        "[HttpDelete] public void ActionD() {}",
        "[HttpPatch] public void ActionE() {}",
    ];
    let expected = BTreeSet::from([
        "ActionA".to_string(),
        "ActionB".to_string(),
        "ActionC".to_string(),
        "ActionD".to_string(),
        "ActionE".to_string(),
    ]);
    for reverse in [false, true] {
        for constructor_position in [0, 2, 5] {
            for constructor in [
                "public Endpoint() {}",
                "public Endpoint(int value) { var local = value; }",
            ] {
                let mut members = methods.to_vec();
                if reverse {
                    members.reverse();
                }
                members.insert(constructor_position, constructor);
                let source = format!(
                    "using Microsoft.AspNetCore.Mvc; public class Endpoint : ControllerBase {{ {} }}",
                    members.join("\n")
                );
                let compiler = compile(&source);
                let actions = compiler
                    .semantic_edges
                    .iter()
                    .filter(|edge| edge.relation == SemanticRelation::ControllerAction)
                    .map(|edge| edge.object.name.clone())
                    .collect::<BTreeSet<_>>();
                assert_eq!(actions, expected, "source: {source}");
            }
        }
    }
}

#[test]
fn compilation_identity_overloads_keep_their_own_attributes() {
    let compiler = compile(
        r#"
using Microsoft.AspNetCore.Mvc;
public class Endpoint : ControllerBase {
    public Endpoint() {}
    [HttpGet] public void Read() {}
    [HttpPost] public void Read(int value) {}
    [HttpDelete] public void Last() {}
}
"#,
    );
    let classes = resolved(&compiler);
    let class = classes
        .iter()
        .find(|class| class.owner == "Endpoint")
        .unwrap();
    let actual = class
        .methods
        .iter()
        .map(|method| {
            (
                method.name.as_str(),
                method
                    .attributes
                    .iter()
                    .map(|attr| attr.written.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("Read", vec!["HttpGet"]),
            ("Read", vec!["HttpPost"]),
            ("Last", vec!["HttpDelete"])
        ]
    );
}

#[test]
fn compilation_identity_method_generics_cannot_change_framework_ancestry() {
    let cases = [
        ("Microsoft.AspNetCore.Mvc.ControllerBase", "controller"),
        ("Microsoft.AspNetCore.SignalR.Hub", "hub"),
        ("AutoMapper.Profile", "profile"),
        ("Microsoft.EntityFrameworkCore.DbContext", "context"),
    ];
    let generic_methods = "protected void Nf<T>() {} protected void Nf<T, U>(int value) {} protected void Other<A, B, C>() {}";
    for (framework, role) in cases {
        for partial in [false, true] {
            for methods_in_base_part in [false, true] {
                let modifier = if partial { "partial " } else { "" };
                let base_methods = if !partial || methods_in_base_part {
                    generic_methods
                } else {
                    ""
                };
                let other_part = if partial {
                    format!(
                        "public partial class Root {{ {} }}",
                        if methods_in_base_part {
                            ""
                        } else {
                            generic_methods
                        }
                    )
                } else {
                    String::new()
                };
                let source = format!(
                    "namespace Bases {{ public {modifier}class Root : global::{framework} {{ {base_methods} }} {other_part} }}
                     namespace Descendants {{ public class Middle : global::Bases.Root {{ }} public class Leaf : Middle {{ }} }}"
                );
                let compiler = compile(&source);
                let classes = resolved(&compiler);
                for name in ["Root", "Middle", "Leaf"] {
                    let class = classes.iter().find(|class| class.owner == name).unwrap();
                    let admitted = match role {
                        "controller" => is_resolved_aspnet_controller(class),
                        "hub" => is_resolved_signalr_hub(class),
                        "profile" => is_resolved_automapper_profile(class),
                        "context" => is_resolved_dbcontext(class),
                        _ => unreachable!(),
                    };
                    assert!(admitted, "{role}/{name}: {source}; {:?}", class.ancestry);
                }
            }
        }
    }
}

#[test]
fn compilation_identity_real_class_arity_survives_method_generics() {
    for parameters in ["T", "T, U", "T, U, V"] {
        let source = format!(
            "public class Root<{parameters}> : global::Microsoft.AspNetCore.Mvc.ControllerBase {{ public void M<A, B, C, D>() {{}} }}
             public class Leaf<{parameters}> : Root<{parameters}> {{}}"
        );
        let classes = resolved(&compile(&source));
        for name in ["Root", "Leaf"] {
            let class = classes.iter().find(|class| class.owner == name).unwrap();
            assert!(is_resolved_aspnet_controller(class), "{name}: {source}");
        }
    }
}
