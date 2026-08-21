// src/tests/ir/layers/rust.rs
//
// Unit tests for the Rust language layer (Layer 2).

#[cfg(test)]
mod rust_layer_tests {
    use crate::ir::layers::rust::{RustLayer, SelfKind};

    #[test]
    fn test_extract_impl_relationships_trait_impl() {
        let input = "impl<T> Repository<T> for PostgresRepo";
        let (self_type, traits) = RustLayer::extract_impl_relationships(input);
        assert_eq!(self_type, Some("PostgresRepo".to_string()));
        assert_eq!(traits, vec!["Repository".to_string()]);
    }

    #[test]
    fn test_extract_impl_relationships_simple_trait_impl() {
        let input = "impl Display for MyStruct";
        let (self_type, traits) = RustLayer::extract_impl_relationships(input);
        assert_eq!(self_type, Some("MyStruct".to_string()));
        assert_eq!(traits, vec!["Display".to_string()]);
    }

    #[test]
    fn test_extract_impl_relationships_inherent_impl() {
        let input = "impl MyStruct";
        let (self_type, traits) = RustLayer::extract_impl_relationships(input);
        assert_eq!(self_type, None);
        assert!(traits.is_empty());
    }

    #[test]
    fn test_extract_impl_relationships_multiple_traits() {
        let input = "impl TraitA for TypeA";
        let (self_type, traits) = RustLayer::extract_impl_relationships(input);
        assert_eq!(self_type, Some("TypeA".to_string()));
        assert_eq!(traits, vec!["TraitA".to_string()]);
    }

    #[test]
    fn test_extract_self_kind_ref() {
        assert_eq!(RustLayer::extract_self_kind("&self"), SelfKind::Ref);
    }

    #[test]
    fn test_extract_self_kind_ref_mut() {
        assert_eq!(RustLayer::extract_self_kind("&mut self"), SelfKind::RefMut);
    }

    #[test]
    fn test_extract_self_kind_owned() {
        assert_eq!(RustLayer::extract_self_kind("self"), SelfKind::Owned);
    }

    #[test]
    fn test_extract_self_kind_none() {
        assert_eq!(RustLayer::extract_self_kind("id: u32"), SelfKind::None);
    }

    #[test]
    fn test_extract_self_kind_none_empty() {
        assert_eq!(RustLayer::extract_self_kind(""), SelfKind::None);
    }

    #[test]
    fn test_extract_derives_simple() {
        let source = r#"#[derive(Debug, Clone)]
pub struct MyStruct {
    field: i32,
}"#;
        let type_start = source.find("pub struct").unwrap();
        let derives = RustLayer::extract_derives(source, type_start);
        assert_eq!(derives, vec!["Debug", "Clone"]);
    }

    #[test]
    fn test_extract_derives_empty() {
        let source = r#"pub struct MyStruct {
    field: i32,
}"#;
        let derives = RustLayer::extract_derives(source, 0);
        assert!(derives.is_empty());
    }

    #[test]
    fn test_extract_derives_complex() {
        let source = r#"#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Status {
    Active,
    Inactive,
}"#;
        let type_start = source.find("pub enum").unwrap();
        let derives = RustLayer::extract_derives(source, type_start);
        assert_eq!(
            derives,
            vec!["Serialize", "Deserialize", "Clone", "Debug", "PartialEq"]
        );
    }

    #[test]
    fn test_extract_cfg_present() {
        let source = r#"#[cfg(feature = "unstable")]
pub struct UnstableFeature {
    field: i32,
}"#;
        let type_start = source.find("pub struct").unwrap();
        let cfg = RustLayer::extract_cfg(source, type_start);
        assert_eq!(cfg, Some(r#"feature = "unstable""#.to_string()));
    }

    #[test]
    fn test_extract_cfg_absent() {
        let source = r#"pub struct Simple {
    field: i32,
}"#;
        let cfg = RustLayer::extract_cfg(source, 0);
        assert_eq!(cfg, None);
    }

    #[test]
    fn test_extract_cfg_multiple_cfg_annotations() {
        let source = r#"#[cfg(any(target_os = "linux", target_os = "macos"))]
pub struct PlatformSpecific {
    field: i32,
}"#;
        let type_start = source.find("pub struct").unwrap();
        let cfg = RustLayer::extract_cfg(source, type_start);
        assert!(cfg.is_some(), "should find cfg attribute");
        let cfg_str = cfg.unwrap();
        assert!(cfg_str.contains("target_os"), "should contain target_os");
        assert!(cfg_str.contains("linux"), "should contain linux");
    }

    #[test]
    fn test_extract_generic_params_simple() {
        let input = "MyStruct<T>";
        let params = RustLayer::extract_generic_params(input);
        assert_eq!(params, Some("<T>".to_string()));
    }

    #[test]
    fn test_extract_generic_params_multiple() {
        let input = "Repository<T, U>";
        let params = RustLayer::extract_generic_params(input);
        assert_eq!(params, Some("<T, U>".to_string()));
    }

    #[test]
    fn test_extract_generic_params_nested() {
        let input = "Cache<HashMap<K, V>>";
        let params = RustLayer::extract_generic_params(input);
        assert_eq!(params, Some("<HashMap<K, V>>".to_string()));
    }

    #[test]
    fn test_extract_generic_params_none() {
        let input = "MyStruct";
        let params = RustLayer::extract_generic_params(input);
        assert_eq!(params, None);
    }

    #[test]
    fn test_extract_generic_params_with_where() {
        let input = "MyStruct<T>";
        let params = RustLayer::extract_generic_params(input);
        assert_eq!(params, Some("<T>".to_string()));
    }
}
