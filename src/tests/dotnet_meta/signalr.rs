// src/tests/dotnet_meta/signalr.rs
//
// Tests for SignalR extraction.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::compression::Fidelity;
    use crate::dotnet_meta::signalr::extract_signalr;

    #[test]
    fn test_extracts_hub() {
        let source = r#"
            public class ChatHub : Hub<IChatClient> {
                public async Task SendMessage(string message) {
                    await Clients.All.ReceiveMessage(message);
                }
            }
        "#;
        let result = extract_signalr(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φhub:ChatHub")));
        assert!(block.lines.iter().any(|l| l.contains("IChatClient")));
    }

    #[test]
    fn test_extracts_hub_methods() {
        let source = r#"
            public class NotificationHub : Hub {
                public async Task SendToUser(string userId, string message) {
                    await Clients.User(userId).ReceiveNotification(message);
                }
            }
        "#;
        let result = extract_signalr(source, Fidelity::Medium);
        assert!(result.is_some());
        let block = result.unwrap();
        assert!(block.lines.iter().any(|l| l.contains("Φmethod:SendToUser")));
    }

    #[test]
    fn test_returns_none_for_non_hub() {
        let source = r#"
            public class Service {
                public void DoWork() { }
            }
        "#;
        let result = extract_signalr(source, Fidelity::Medium);
        assert!(result.is_none());
    }
}
