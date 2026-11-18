// src/tests/dotnet_meta/efcore.rs
//
// Tests for EF Core extraction.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::compression::Fidelity;
    use crate::dotnet_meta::efcore::extract_efcore;

    #[test]
    fn test_extracts_dbcontext() {
        let source = r#"
            public class AppDbContext : DbContext {
                public DbSet<User> Users { get; set; }
                public DbSet<Order> Orders { get; set; }
            }
        "#;
        let analysis =
            crate::dotnet_meta::efcore::analyze_authorized_efcore(source, Fidelity::Medium)
                .unwrap();
        let targets: Vec<_> = analysis
            .semantic_edges
            .iter()
            .map(|edge| {
                assert_eq!(
                    edge.relation,
                    crate::layers::meta::semantic::SemanticRelation::HasEntity
                );
                assert_eq!(edge.subject.name, "AppDbContext");
                edge.object.name.as_str()
            })
            .collect();
        assert_eq!(targets, vec!["User", "Order"]);
        let result = extract_efcore(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φef:AppDbContext")));
        assert!(block.lines.iter().any(|l| l.contains("Φdbset:Users")));
        assert!(block.lines.iter().any(|l| l.contains("Φdbset:Orders")));
    }

    #[test]
    fn test_returns_none_for_non_dbcontext() {
        let source = r#"
            public class User {
                public string Name { get; set; }
            }
        "#;
        let result = extract_efcore(source, Fidelity::Medium);
        assert!(result.is_none());
    }
}
