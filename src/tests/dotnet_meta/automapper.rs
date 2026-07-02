// src/tests/dotnet_meta/automapper.rs
//
// Tests for AutoMapper extraction.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::automapper::extract_automapper;
    use crate::compression::Fidelity;

    #[test]
    fn test_extracts_profile() {
        let source = r#"
            public class UserProfile : Profile {
                public UserProfile() {
                    CreateMap<User, UserDto>();
                    CreateMap<Order, OrderDto>();
                }
            }
        "#;
        let result = extract_automapper(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φmap:UserProfile")));
        assert!(block.lines.iter().any(|l| l.contains("Φmapfrom:User → UserDto")));
    }

    #[test]
    fn test_returns_none_for_non_profile() {
        let source = r#"
            public class Service {
                public void DoWork() { }
            }
        "#;
        let result = extract_automapper(source, Fidelity::Medium);
        assert!(result.is_none());
    }
}