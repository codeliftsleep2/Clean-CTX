// src/tests/dotnet_meta/serialization.rs
//
// Tests for serialization extraction.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::serialization::extract_serialization;
    use crate::compression::Fidelity;

    #[test]
    fn test_extracts_json_properties() {
        let source = r#"
            public class User {
                [JsonPropertyName("user_id")]
                public int Id { get; set; }
                
                [JsonPropertyName("full_name")]
                public string Name { get; set; }
            }
        "#;
        let result = extract_serialization(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φprop:user_id")));
        assert!(block.lines.iter().any(|l| l.contains("Φprop:full_name")));
    }

    #[test]
    fn test_returns_none_for_no_serialization() {
        let source = r#"
            public class Person {
                public string Name { get; set; }
            }
        "#;
        let result = extract_serialization(source, Fidelity::Medium);
        assert!(result.is_none());
    }
}