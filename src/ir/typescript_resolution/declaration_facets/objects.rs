//! Existing field-filtered Object acquisition and the approved immediate resolve origin.
use super::*;
impl Demand<'_> {
    pub(super) fn object(
        &mut self,
        node: tree_sitter::Node<'_>,
        file: &Path,
        source: &str,
    ) -> Vec<(String, ExpressionFacet)> {
        let mut cursor = node.walk();
        let mut fields = Vec::new();
        for property in node
            .named_children(&mut cursor)
            .filter(|node| node.kind() != "comment")
        {
            if property.kind() == "pair" {
                if let (Some(key), Some(value)) = (
                    property.child_by_field_name("key"),
                    property.child_by_field_name("value"),
                ) {
                    if key.kind() == "computed_property_name" {
                        fields.push((String::new(), self.expression(value, file, source)));
                        continue;
                    }
                    let raw = key.utf8_text(source.as_bytes()).unwrap_or_default();
                    let key = if key.kind() == "string" {
                        super::super::string_value::decode(raw).unwrap_or_default()
                    } else {
                        raw.into()
                    };
                    if self.fields.contains(&key) {
                        let expression = if key == "resolve" && value.kind() == "object" {
                            self.resolve_object(node, property, value, file, source)
                        } else {
                            self.expression(value, file, source)
                        };
                        if super::super::arrow_selection::demand_origin(&key)
                            && let Some((proof, sites)) = super::super::arrow_selection::capture(
                                self.resolver,
                                file,
                                source,
                                value,
                            )
                        {
                            self.declaration(&proof.target);
                            for site in sites {
                                self.occurrences
                                    .insert((site.source.clone(), site.start, site.end), site);
                            }
                            self.selections.push(proof);
                        }
                        fields.push((key, expression));
                    }
                }
            } else if property.kind() == "shorthand_property_identifier" {
                let key = property.utf8_text(source.as_bytes()).unwrap_or_default();
                if self.fields.contains(key) {
                    fields.push((key.into(), self.expression(property, file, source)));
                }
            } else {
                fields.push((String::new(), self.expression(property, file, source)));
            }
        }
        fields
    }
}
