use super::*;

#[test]
fn csharp_where_clause_is_not_the_canonical_return_type() {
    const SOURCE: &str =
        "public class Factory { public T Create<T>() where T : new() { return new T(); } }";
    for (fidelity, expected_return) in [(Fidelity::Medium, "$v"), (Fidelity::High, "T")] {
        let ir = compile_cs(SOURCE, fidelity);
        let facts = method(&ir, "Create<T>");
        assert_eq!(facts.return_type, expected_return, "{fidelity:?}");
        assert!(facts.params.is_empty());
    }
}

#[test]
fn csharp_parameter_attribute_is_not_part_of_the_canonical_type() {
    let ir = compile_cs(
        "public class Consumer { public Consumer([FromServices] IFooService service) {} }",
        Fidelity::High,
    );
    let facts = method(&ir, "Consumer");
    assert_eq!(facts.params, ["service"]);
    assert_eq!(facts.param_types, ["IFooService"]);
}
