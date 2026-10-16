use super::{DecoratorKind, collect_field_decorators};

#[test]
fn setter_and_definite_assignment_names_come_from_member_declarations() {
    let fields = collect_field_decorators(
        r#"{
            @Input('publicName') set value(next: string) {}
            @Input({ required: true }) public readonly item!: Item;
            @Output() changed!: EventEmitter<void>;
        }"#,
    );
    assert!(fields.iter().any(|(kind, alias, name)| {
        *kind == DecoratorKind::Input && alias.as_deref() == Some("publicName") && name == "value"
    }));
    assert!(
        fields
            .iter()
            .any(|(kind, _, name)| { *kind == DecoratorKind::Input && name == "item" })
    );
    assert!(
        fields
            .iter()
            .any(|(kind, _, name)| { *kind == DecoratorKind::Output && name == "changed" })
    );
    assert!(
        fields
            .iter()
            .all(|(_, _, name)| { name != "?" && name != "set" && !name.ends_with('!') })
    );
}

#[test]
fn malformed_decorated_member_does_not_publish_junk_identity() {
    let fields = collect_field_decorators("{ @Input() !: string; }");
    assert!(fields.is_empty(), "{fields:?}");
}
