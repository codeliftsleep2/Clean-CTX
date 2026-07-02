// src/tests/dotnet_meta/general.rs
//
// Tests for general .NET patterns (services, DI, validation, identity, caching, logging).

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::general::extract_general;
    use crate::compression::Fidelity;

    #[test]
    fn test_extracts_services() {
        let source = r#"
            public interface IUserService {
                Task<User> GetUserAsync(int id);
            }
            
            public class UserService : IUserService {
                public async Task<User> GetUserAsync(int id) {
                    return null;
                }
            }
        "#;
        let result = extract_general(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φsvc:IUserService")));
        assert!(block.lines.iter().any(|l| l.contains("Φsvc:UserService")));
    }

    #[test]
    fn test_extracts_di_registrations() {
        let source = r#"
            public class Startup {
                public void ConfigureServices(IServiceCollection services) {
                    services.AddScoped<IUserService, UserService>();
                    services.AddSingleton<ICacheService, MemoryCacheService>();
                }
            }
        "#;
        let result = extract_general(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φdi:IUserService")));
        assert!(block.lines.iter().any(|l| l.contains("Φdi:ICacheService")));
    }

    #[test]
    fn test_extracts_validation() {
        let source = r#"
            public class UserValidator : AbstractValidator<User> {
                public UserValidator() {
                    RuleFor(x => x.Email).NotEmpty().EmailAddress();
                    RuleFor(x => x.Age).GreaterThan(18);
                }
            }
        "#;
        let result = extract_general(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φvalid:User")));
        assert!(block.lines.iter().any(|l| l.contains("Φrule:Email")));
    }

    #[test]
    fn test_extracts_logging() {
        let source = r#"
            public class MyService {
                private readonly ILogger<MyService> _logger;
            }
        "#;
        let result = extract_general(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φlog:ILogger<MyService>")));
    }

    #[test]
    fn test_extracts_caching() {
        let source = r#"
            public class CacheService {
                private readonly IMemoryCache _cache;
            }
        "#;
        let result = extract_general(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φcache:IMemoryCache")));
    }
}