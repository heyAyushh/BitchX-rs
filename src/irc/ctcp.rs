pub struct CtcpMessage {
    pub command: String,
    pub params: Option<String>,
}

const CTCP_DELIM: char = '\x01';

impl CtcpMessage {
    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim_matches(CTCP_DELIM);
        if trimmed.is_empty() || !text.starts_with(CTCP_DELIM) || !text.ends_with(CTCP_DELIM) {
            return None;
        }
        let (command, params) = match trimmed.split_once(' ') {
            Some((cmd, rest)) => (cmd, Some(rest.to_string())),
            None => (trimmed, None),
        };
        Some(Self {
            command: command.to_ascii_uppercase(),
            params,
        })
    }

    pub fn is_ctcp(text: &str) -> bool {
        text.len() >= 2 && text.starts_with(CTCP_DELIM) && text.ends_with(CTCP_DELIM)
    }

    pub fn encode(command: &str, params: Option<&str>) -> String {
        match params {
            Some(p) => format!("\x01{} {}\x01", command.to_ascii_uppercase(), p),
            None => format!("\x01{}\x01", command.to_ascii_uppercase()),
        }
    }

    pub fn version_reply() -> String {
        Self::encode("VERSION", Some("BitchX 2.0.0-rs - relay (リレー) release candidate"))
    }

    pub fn ping_reply(token: &str) -> String {
        Self::encode("PING", Some(token))
    }

    pub fn time_reply() -> String {
        let now = chrono::Utc::now()
            .format("%a %b %d %H:%M:%S %Y UTC")
            .to_string();
        Self::encode("TIME", Some(&now))
    }

    pub fn clientinfo_reply() -> String {
        Self::encode(
            "CLIENTINFO",
            Some("ACTION CLIENTINFO DCC FINGER PING TIME VERSION"),
        )
    }

    pub fn action(text: &str) -> String {
        Self::encode("ACTION", Some(text))
    }

    pub fn finger_reply(username: &str) -> String {
        Self::encode("FINGER", Some(username))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let msg = CtcpMessage::parse("\x01VERSION\x01").unwrap();
        assert_eq!(msg.command, "VERSION");
        assert!(msg.params.is_none());
    }

    #[test]
    fn test_parse_ping_with_token() {
        let msg = CtcpMessage::parse("\x01PING 12345\x01").unwrap();
        assert_eq!(msg.command, "PING");
        assert_eq!(msg.params.as_deref(), Some("12345"));
    }

    #[test]
    fn test_parse_action() {
        let msg = CtcpMessage::parse("\x01ACTION waves hello\x01").unwrap();
        assert_eq!(msg.command, "ACTION");
        assert_eq!(msg.params.as_deref(), Some("waves hello"));
    }

    #[test]
    fn test_parse_empty() {
        assert!(CtcpMessage::parse("").is_none());
    }

    #[test]
    fn test_parse_no_delimiters() {
        assert!(CtcpMessage::parse("VERSION").is_none());
    }

    #[test]
    fn test_parse_only_delimiters() {
        assert!(CtcpMessage::parse("\x01\x01").is_none());
    }

    #[test]
    fn test_parse_case_insensitive_command() {
        let msg = CtcpMessage::parse("\x01version\x01").unwrap();
        assert_eq!(msg.command, "VERSION");
    }

    #[test]
    fn test_is_ctcp() {
        assert!(CtcpMessage::is_ctcp("\x01VERSION\x01"));
        assert!(CtcpMessage::is_ctcp("\x01PING 123\x01"));
        assert!(!CtcpMessage::is_ctcp("hello"));
        assert!(!CtcpMessage::is_ctcp("\x01"));
        assert!(!CtcpMessage::is_ctcp(""));
    }

    #[test]
    fn test_encode_no_params() {
        let encoded = CtcpMessage::encode("VERSION", None);
        assert_eq!(encoded, "\x01VERSION\x01");
    }

    #[test]
    fn test_encode_with_params() {
        let encoded = CtcpMessage::encode("PING", Some("12345"));
        assert_eq!(encoded, "\x01PING 12345\x01");
    }

    #[test]
    fn test_encode_round_trip() {
        let original = CtcpMessage::encode("PING", Some("token123"));
        let parsed = CtcpMessage::parse(&original).unwrap();
        assert_eq!(parsed.command, "PING");
        assert_eq!(parsed.params.as_deref(), Some("token123"));
    }

    #[test]
    fn test_version_reply() {
        let reply = CtcpMessage::version_reply();
        assert!(reply.starts_with('\x01'));
        assert!(reply.ends_with('\x01'));
        assert!(reply.contains("VERSION"));
        assert!(reply.contains("BitchX"));
    }

    #[test]
    fn test_ping_reply() {
        let reply = CtcpMessage::ping_reply("99999");
        let parsed = CtcpMessage::parse(&reply).unwrap();
        assert_eq!(parsed.command, "PING");
        assert_eq!(parsed.params.as_deref(), Some("99999"));
    }

    #[test]
    fn test_time_reply() {
        let reply = CtcpMessage::time_reply();
        let parsed = CtcpMessage::parse(&reply).unwrap();
        assert_eq!(parsed.command, "TIME");
        assert!(parsed.params.is_some());
    }

    #[test]
    fn test_clientinfo_reply() {
        let reply = CtcpMessage::clientinfo_reply();
        let parsed = CtcpMessage::parse(&reply).unwrap();
        assert_eq!(parsed.command, "CLIENTINFO");
        let params = parsed.params.unwrap();
        assert!(params.contains("VERSION"));
        assert!(params.contains("PING"));
        assert!(params.contains("ACTION"));
    }

    #[test]
    fn test_action() {
        let action = CtcpMessage::action("dances around");
        let parsed = CtcpMessage::parse(&action).unwrap();
        assert_eq!(parsed.command, "ACTION");
        assert_eq!(parsed.params.as_deref(), Some("dances around"));
    }

    #[test]
    fn test_finger_reply() {
        let reply = CtcpMessage::finger_reply("alice");
        let parsed = CtcpMessage::parse(&reply).unwrap();
        assert_eq!(parsed.command, "FINGER");
        assert_eq!(parsed.params.as_deref(), Some("alice"));
    }
}
