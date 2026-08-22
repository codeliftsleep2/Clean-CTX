// src/tests/dotnet_meta/detect.rs
//
// Tests for .NET file detection heuristic.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::detect::is_dotnet_file;

    #[test]
    fn test_detects_aspnet_controller() {
        let source = r#"
            [ApiController]
            [Route("api/[controller]")]
            public class UserController : ControllerBase {
                [HttpGet]
                public IActionResult Get() { return Ok(); }
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_efcore_dbcontext() {
        let source = r#"
            public class AppDbContext : DbContext {
                public DbSet<User> Users { get; set; }
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_signalr_hub() {
        let source = r#"
            public class ChatHub : Hub<IChatClient> {
                public async Task SendMessage(string message) {
                    await Clients.All.ReceiveMessage(message);
                }
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_automapper_profile() {
        let source = r#"
            public class UserProfile : Profile {
                public UserProfile() {
                    CreateMap<User, UserDto>();
                }
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_fluent_validation() {
        let source = r#"
            public class UserValidator : AbstractValidator<User> {
                public UserValidator() {
                    RuleFor(x => x.Email).NotEmpty().EmailAddress();
                }
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_identity() {
        let source = r#"
            public class AuthService {
                private readonly UserManager<ApplicationUser> _userManager;
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_caching() {
        let source = r#"
            public class CacheService {
                private readonly IMemoryCache _cache;
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_logging() {
        let source = r#"
            public class MyService {
                private readonly ILogger<MyService> _logger;
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_detects_background_jobs() {
        let source = r#"
            public class JobService {
                public void ScheduleJob() {
                    BackgroundJob.Enqueue(() => DoWork());
                }
            }
        "#;
        assert!(is_dotnet_file(source));
    }

    #[test]
    fn test_rejects_plain_csharp() {
        let source = r#"
            public class Utility {
                public string HelperMethod(string input) {
                    return input.ToUpper();
                }
            }
        "#;
        assert!(!is_dotnet_file(source));
    }

    #[test]
    fn test_rejects_simple_poco() {
        let source = r#"
            public class Person {
                public string Name { get; set; }
                public int Age { get; set; }
            }
        "#;
        assert!(!is_dotnet_file(source));
    }
}
