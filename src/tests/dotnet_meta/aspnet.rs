// src/tests/dotnet_meta/aspnet.rs
//
// Tests for ASP.NET Core extraction.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::aspnet::extract_aspnet;
    use crate::compression::Fidelity;

    #[test]
    fn test_extracts_controller() {
        let source = r#"
            [ApiController]
            [Route("api/[controller]")]
            public class UserController : ControllerBase {
                [HttpGet]
                public IActionResult Get() { return Ok(); }
            }
        "#;
        let result = extract_aspnet(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(!block.lines.is_empty());
        assert!(block.lines.iter().any(|l| l.contains("Φctrl:UserController")));
        assert!(block.lines.iter().any(|l| l.contains("Φapi:UserController")));
    }

    #[test]
    fn test_extracts_actions() {
        let source = r#"
            [ApiController]
            [Route("api/[controller]")]
            public class UserController : ControllerBase {
                [HttpGet("{id}")]
                public IActionResult GetById(int id) { return Ok(); }
                
                [HttpPost]
                public IActionResult Create([FromBody] User user) { return Created(); }
            }
        "#;
        let result = extract_aspnet(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φaction:GET GetById")));
        assert!(block.lines.iter().any(|l| l.contains("Φaction:POST Create")));
    }

    #[test]
    fn test_extracts_authorization() {
        let source = r#"
            [Authorize(Roles = "Admin")]
            public class AdminController : ControllerBase {
            }
        "#;
        let result = extract_aspnet(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φauth:Admin")));
    }

    #[test]
    fn test_returns_none_for_non_controller() {
        let source = r#"
            public class Service {
                public void DoWork() { }
            }
        "#;
        let result = extract_aspnet(source, Fidelity::Medium);
        assert!(result.is_none());
    }
}