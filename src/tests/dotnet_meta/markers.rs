// src/tests/dotnet_meta/markers.rs
//
// Tests for .NET marker construction and expansion.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::markers::{
        PhiLineKind, build_action_line, build_controller_line, build_ef_line, build_hub_line,
    };

    #[test]
    fn test_controller_line() {
        let line = build_controller_line("UserController", Some("api/users"));
        assert_eq!(line, "Φctrl:UserController [api/users]");
    }

    #[test]
    fn test_controller_line_without_route() {
        let line = build_controller_line("UserController", None);
        assert_eq!(line, "Φctrl:UserController");
    }

    #[test]
    fn test_action_line() {
        let line = build_action_line("GET", "GetById", "int id", Some("IActionResult"));
        assert_eq!(line, "Φaction:GET GetById(int id) → IActionResult");
    }

    #[test]
    fn test_ef_line() {
        let line = build_ef_line("AppDbContext");
        assert_eq!(line, "Φef:AppDbContext");
    }

    #[test]
    fn test_hub_line_with_client() {
        let line = build_hub_line("ChatHub", Some("IChatClient"));
        assert_eq!(line, "Φhub:ChatHub [IChatClient]");
    }

    #[test]
    fn test_phi_line_kind_from_token() {
        assert_eq!(
            PhiLineKind::from_token("Φctrl"),
            Some(PhiLineKind::Controller)
        );
        assert_eq!(PhiLineKind::from_token("Φef"), Some(PhiLineKind::DbContext));
        assert_eq!(PhiLineKind::from_token("Φhub"), Some(PhiLineKind::Hub));
        assert_eq!(PhiLineKind::from_token("Φunknown"), None);
    }

    #[test]
    fn test_expand_phi_in_line() {
        let line = "Φctrl:UserController Φef:AppDbContext";
        let expanded = crate::dotnet_meta::markers::expand_phi_in_line(line);
        assert!(expanded.contains("[Controller]"));
        assert!(expanded.contains("[DbContext]"));
    }
}
