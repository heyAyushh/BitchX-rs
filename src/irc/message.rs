use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Prefix {
    Server(String),
    User {
        nick: String,
        user: Option<String>,
        host: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrcMessage {
    pub prefix: Option<Prefix>,
    pub command: String,
    pub params: Vec<String>,
}

impl IrcMessage {
    pub fn parse(line: &str) -> Result<Self, crate::error::BitchXError> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            return Err(crate::error::BitchXError::Parse("empty input".into()));
        }

        let mut rest = line;

        let prefix_str = if rest.starts_with(':') {
            let end = rest
                .find(' ')
                .ok_or_else(|| crate::error::BitchXError::Parse("prefix with no command".into()))?;
            let p = &rest[1..end];
            rest = &rest[end + 1..];
            Some(p)
        } else {
            None
        };

        rest = rest.trim_start();
        if rest.is_empty() {
            return Err(crate::error::BitchXError::Parse("no command found".into()));
        }

        let mut params = Vec::new();
        let command;

        if let Some(space_pos) = rest.find(' ') {
            command = rest[..space_pos].to_uppercase();
            rest = &rest[space_pos + 1..];

            while !rest.is_empty() {
                if let Some(trailing) = rest.strip_prefix(':') {
                    params.push(trailing.to_string());
                    break;
                }
                if let Some(pos) = rest.find(' ') {
                    params.push(rest[..pos].to_string());
                    rest = &rest[pos + 1..];
                } else {
                    params.push(rest.to_string());
                    break;
                }
            }
        } else {
            command = rest.to_uppercase();
        }

        let is_numeric = command.len() == 3 && command.chars().all(|c| c.is_ascii_digit());
        let prefix = prefix_str.map(|s| parse_prefix(s, is_numeric));

        Ok(IrcMessage {
            prefix,
            command,
            params,
        })
    }

    pub fn trailing(&self) -> Option<&str> {
        self.params.last().map(|s| s.as_str())
    }

    pub fn nick(&self) -> Option<&str> {
        match &self.prefix {
            Some(Prefix::User { nick, .. }) => Some(nick.as_str()),
            _ => None,
        }
    }

    pub fn user(&self) -> Option<&str> {
        match &self.prefix {
            Some(Prefix::User { user, .. }) => user.as_deref(),
            _ => None,
        }
    }

    pub fn host(&self) -> Option<&str> {
        match &self.prefix {
            Some(Prefix::User { host, .. }) => host.as_deref(),
            _ => None,
        }
    }

    pub fn to_raw(&self) -> String {
        let mut out = String::new();
        if let Some(ref prefix) = self.prefix {
            out.push(':');
            match prefix {
                Prefix::Server(s) => out.push_str(s),
                Prefix::User { nick, user, host } => {
                    out.push_str(nick);
                    if let Some(u) = user {
                        out.push('!');
                        out.push_str(u);
                    }
                    if let Some(h) = host {
                        out.push('@');
                        out.push_str(h);
                    }
                }
            }
            out.push(' ');
        }
        out.push_str(&self.command);
        for (i, param) in self.params.iter().enumerate() {
            out.push(' ');
            if i == self.params.len() - 1 {
                out.push(':');
            }
            out.push_str(param);
        }
        out
    }
}

impl fmt::Display for IrcMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_raw())
    }
}

fn parse_prefix(s: &str, numeric_command: bool) -> Prefix {
    if let Some(bang_pos) = s.find('!') {
        let nick = s[..bang_pos].to_string();
        let rest = &s[bang_pos + 1..];
        if let Some(at_pos) = rest.find('@') {
            Prefix::User {
                nick,
                user: Some(rest[..at_pos].to_string()),
                host: Some(rest[at_pos + 1..].to_string()),
            }
        } else {
            Prefix::User {
                nick,
                user: Some(rest.to_string()),
                host: None,
            }
        }
    } else if let Some(at_pos) = s.find('@') {
        Prefix::User {
            nick: s[..at_pos].to_string(),
            user: None,
            host: Some(s[at_pos + 1..].to_string()),
        }
    } else if s.contains('.') || numeric_command {
        Prefix::Server(s.to_string())
    } else {
        Prefix::User {
            nick: s.to_string(),
            user: None,
            host: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_ping() {
        let msg = IrcMessage::parse("PING :server.name").unwrap();
        assert_eq!(msg.prefix, None);
        assert_eq!(msg.command, "PING");
        assert_eq!(msg.params, vec!["server.name"]);
    }

    #[test]
    fn parse_privmsg_with_prefix() {
        let msg = IrcMessage::parse(":nick!user@host PRIVMSG #channel :Hello world").unwrap();
        assert_eq!(
            msg.prefix,
            Some(Prefix::User {
                nick: "nick".into(),
                user: Some("user".into()),
                host: Some("host".into()),
            })
        );
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#channel", "Hello world"]);
    }

    #[test]
    fn parse_numeric_reply_001() {
        let msg = IrcMessage::parse(":server 001 nick :Welcome to the IRC Network").unwrap();
        assert_eq!(msg.prefix, Some(Prefix::Server("server".into())));
        assert_eq!(msg.command, "001");
        assert_eq!(msg.params, vec!["nick", "Welcome to the IRC Network"]);
    }

    #[test]
    fn parse_numeric_reply_353() {
        let msg = IrcMessage::parse(":server.name 353 mynick = #channel :nick1 nick2 @op +voice")
            .unwrap();
        assert_eq!(msg.prefix, Some(Prefix::Server("server.name".into())));
        assert_eq!(msg.command, "353");
        assert_eq!(
            msg.params,
            vec!["mynick", "=", "#channel", "nick1 nick2 @op +voice"]
        );
    }

    #[test]
    fn parse_join_with_user_prefix() {
        let msg = IrcMessage::parse(":nick!user@host JOIN #channel").unwrap();
        assert_eq!(
            msg.prefix,
            Some(Prefix::User {
                nick: "nick".into(),
                user: Some("user".into()),
                host: Some("host".into()),
            })
        );
        assert_eq!(msg.command, "JOIN");
        assert_eq!(msg.params, vec!["#channel"]);
    }

    #[test]
    fn parse_mode_with_multiple_params() {
        let msg = IrcMessage::parse(":server MODE #channel +ov nick1 nick2").unwrap();
        assert_eq!(msg.command, "MODE");
        assert_eq!(msg.params, vec!["#channel", "+ov", "nick1", "nick2"]);
    }

    #[test]
    fn parse_message_no_prefix() {
        let msg = IrcMessage::parse("NOTICE AUTH :*** Looking up your hostname").unwrap();
        assert_eq!(msg.prefix, None);
        assert_eq!(msg.command, "NOTICE");
        assert_eq!(msg.params, vec!["AUTH", "*** Looking up your hostname"]);
    }

    #[test]
    fn parse_trailing_with_spaces() {
        let msg = IrcMessage::parse(":nick PRIVMSG #chan :this is a message with spaces").unwrap();
        assert_eq!(msg.params, vec!["#chan", "this is a message with spaces"]);
    }

    #[test]
    fn round_trip_privmsg() {
        let raw = ":nick!user@host PRIVMSG #channel :Hello world";
        let msg = IrcMessage::parse(raw).unwrap();
        assert_eq!(msg.to_raw(), raw);
    }

    #[test]
    fn round_trip_ping() {
        let raw = "PING :server.name";
        let msg = IrcMessage::parse(raw).unwrap();
        assert_eq!(msg.to_raw(), raw);
    }

    #[test]
    fn round_trip_mode() {
        let raw = ":server MODE #channel +ov nick1 nick2";
        let msg = IrcMessage::parse(raw).unwrap();
        let serialized = msg.to_raw();
        let reparsed = IrcMessage::parse(&serialized).unwrap();
        assert_eq!(msg, reparsed);
    }

    #[test]
    fn error_on_empty_input() {
        assert!(IrcMessage::parse("").is_err());
    }

    #[test]
    fn error_on_whitespace_only() {
        assert!(IrcMessage::parse("   \r\n").is_err());
    }

    #[test]
    fn error_on_prefix_only() {
        assert!(IrcMessage::parse(":prefix").is_err());
    }

    #[test]
    fn parse_quit_with_trailing() {
        let msg = IrcMessage::parse(":nick!user@host QUIT :Gone to lunch").unwrap();
        assert_eq!(msg.command, "QUIT");
        assert_eq!(msg.params, vec!["Gone to lunch"]);
        assert_eq!(msg.trailing(), Some("Gone to lunch"));
    }

    #[test]
    fn parse_kick_with_reason() {
        let msg = IrcMessage::parse(":op!user@host KICK #channel badnick :You are kicked").unwrap();
        assert_eq!(msg.command, "KICK");
        assert_eq!(msg.params, vec!["#channel", "badnick", "You are kicked"]);
    }

    #[test]
    fn nick_extraction() {
        let msg = IrcMessage::parse(":testnick!testuser@testhost PRIVMSG #chan :hi").unwrap();
        assert_eq!(msg.nick(), Some("testnick"));
    }

    #[test]
    fn user_extraction() {
        let msg = IrcMessage::parse(":testnick!testuser@testhost PRIVMSG #chan :hi").unwrap();
        assert_eq!(msg.user(), Some("testuser"));
    }

    #[test]
    fn host_extraction() {
        let msg = IrcMessage::parse(":testnick!testuser@testhost PRIVMSG #chan :hi").unwrap();
        assert_eq!(msg.host(), Some("testhost"));
    }

    #[test]
    fn nick_from_server_prefix_is_none() {
        let msg = IrcMessage::parse(":irc.server.com 001 nick :Welcome").unwrap();
        assert_eq!(msg.nick(), None);
    }

    #[test]
    fn display_matches_to_raw() {
        let msg = IrcMessage::parse(":nick!user@host PRIVMSG #channel :Hello world").unwrap();
        assert_eq!(format!("{}", msg), msg.to_raw());
    }

    #[test]
    fn parse_with_crlf() {
        let msg = IrcMessage::parse("PING :server\r\n").unwrap();
        assert_eq!(msg.command, "PING");
        assert_eq!(msg.params, vec!["server"]);
    }

    #[test]
    fn parse_command_case_insensitive() {
        let msg = IrcMessage::parse("ping :server").unwrap();
        assert_eq!(msg.command, "PING");
    }

    #[test]
    fn parse_nick_only_prefix() {
        let msg = IrcMessage::parse(":justnick QUIT :bye").unwrap();
        assert_eq!(
            msg.prefix,
            Some(Prefix::User {
                nick: "justnick".into(),
                user: None,
                host: None,
            })
        );
        assert_eq!(msg.nick(), Some("justnick"));
    }

    #[test]
    fn trailing_returns_last_param() {
        let msg = IrcMessage::parse(":server 001 nick :Welcome").unwrap();
        assert_eq!(msg.trailing(), Some("Welcome"));
    }

    #[test]
    fn parse_no_params() {
        let msg = IrcMessage::parse("QUIT").unwrap();
        assert_eq!(msg.command, "QUIT");
        assert!(msg.params.is_empty());
    }

    #[test]
    fn parse_empty_trailing() {
        let msg = IrcMessage::parse(":nick PRIVMSG #chan :").unwrap();
        assert_eq!(msg.params, vec!["#chan", ""]);
    }
}
