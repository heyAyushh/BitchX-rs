pub struct IrcCommand;

impl IrcCommand {
    pub fn pass(password: &str) -> String {
        format!("PASS :{password}\r\n")
    }

    pub fn nick(nick: &str) -> String {
        format!("NICK :{nick}\r\n")
    }

    pub fn user(username: &str, realname: &str) -> String {
        format!("USER {username} 0 * :{realname}\r\n")
    }

    pub fn join(channel: &str) -> String {
        format!("JOIN :{channel}\r\n")
    }

    pub fn join_with_key(channel: &str, key: &str) -> String {
        format!("JOIN {channel} :{key}\r\n")
    }

    pub fn part(channel: &str, reason: Option<&str>) -> String {
        match reason {
            Some(r) => format!("PART {channel} :{r}\r\n"),
            None => format!("PART :{channel}\r\n"),
        }
    }

    pub fn privmsg(target: &str, message: &str) -> String {
        format!("PRIVMSG {target} :{message}\r\n")
    }

    pub fn notice(target: &str, message: &str) -> String {
        format!("NOTICE {target} :{message}\r\n")
    }

    pub fn quit(reason: Option<&str>) -> String {
        match reason {
            Some(r) => format!("QUIT :{r}\r\n"),
            None => "QUIT\r\n".to_string(),
        }
    }

    pub fn ping(token: &str) -> String {
        format!("PING :{token}\r\n")
    }

    pub fn pong(token: &str) -> String {
        format!("PONG :{token}\r\n")
    }

    pub fn mode(target: &str, mode: Option<&str>, params: &[&str]) -> String {
        let mut s = format!("MODE {target}");
        if let Some(m) = mode {
            s.push(' ');
            s.push_str(m);
            for p in params {
                s.push(' ');
                s.push_str(p);
            }
        }
        s.push_str("\r\n");
        s
    }

    pub fn kick(channel: &str, nick: &str, reason: Option<&str>) -> String {
        match reason {
            Some(r) => format!("KICK {channel} {nick} :{r}\r\n"),
            None => format!("KICK {channel} :{nick}\r\n"),
        }
    }

    pub fn topic(channel: &str, topic: Option<&str>) -> String {
        match topic {
            Some(t) => format!("TOPIC {channel} :{t}\r\n"),
            None => format!("TOPIC :{channel}\r\n"),
        }
    }

    pub fn invite(nick: &str, channel: &str) -> String {
        format!("INVITE {nick} :{channel}\r\n")
    }

    pub fn whois(nick: &str) -> String {
        format!("WHOIS :{nick}\r\n")
    }

    pub fn who(mask: &str) -> String {
        format!("WHO :{mask}\r\n")
    }

    pub fn list(channel: Option<&str>) -> String {
        match channel {
            Some(c) => format!("LIST :{c}\r\n"),
            None => "LIST\r\n".to_string(),
        }
    }

    pub fn names(channel: &str) -> String {
        format!("NAMES :{channel}\r\n")
    }

    pub fn away(message: Option<&str>) -> String {
        match message {
            Some(m) => format!("AWAY :{m}\r\n"),
            None => "AWAY\r\n".to_string(),
        }
    }

    pub fn cap_req(capabilities: &[&str]) -> String {
        let caps = capabilities.join(" ");
        format!("CAP REQ :{caps}\r\n")
    }

    pub fn cap_end() -> String {
        "CAP END\r\n".to_string()
    }

    pub fn authenticate(data: &str) -> String {
        format!("AUTHENTICATE :{data}\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass() {
        assert_eq!(IrcCommand::pass("secret"), "PASS :secret\r\n");
    }

    #[test]
    fn test_nick() {
        assert_eq!(IrcCommand::nick("coolnick"), "NICK :coolnick\r\n");
    }

    #[test]
    fn test_user() {
        assert_eq!(
            IrcCommand::user("myuser", "My Real Name"),
            "USER myuser 0 * :My Real Name\r\n"
        );
    }

    #[test]
    fn test_join() {
        assert_eq!(IrcCommand::join("#channel"), "JOIN :#channel\r\n");
    }

    #[test]
    fn test_join_with_key() {
        assert_eq!(
            IrcCommand::join_with_key("#channel", "secretkey"),
            "JOIN #channel :secretkey\r\n"
        );
    }

    #[test]
    fn test_part_with_reason() {
        assert_eq!(
            IrcCommand::part("#channel", Some("goodbye")),
            "PART #channel :goodbye\r\n"
        );
    }

    #[test]
    fn test_part_without_reason() {
        assert_eq!(IrcCommand::part("#channel", None), "PART :#channel\r\n");
    }

    #[test]
    fn test_privmsg() {
        assert_eq!(
            IrcCommand::privmsg("#channel", "Hello world"),
            "PRIVMSG #channel :Hello world\r\n"
        );
    }

    #[test]
    fn test_notice() {
        assert_eq!(
            IrcCommand::notice("nick", "You have been noticed"),
            "NOTICE nick :You have been noticed\r\n"
        );
    }

    #[test]
    fn test_quit_with_reason() {
        assert_eq!(
            IrcCommand::quit(Some("Leaving")),
            "QUIT :Leaving\r\n"
        );
    }

    #[test]
    fn test_quit_without_reason() {
        assert_eq!(IrcCommand::quit(None), "QUIT\r\n");
    }

    #[test]
    fn test_ping() {
        assert_eq!(IrcCommand::ping("token123"), "PING :token123\r\n");
    }

    #[test]
    fn test_pong() {
        assert_eq!(IrcCommand::pong("token123"), "PONG :token123\r\n");
    }

    #[test]
    fn test_mode_query() {
        assert_eq!(IrcCommand::mode("#channel", None, &[]), "MODE #channel\r\n");
    }

    #[test]
    fn test_mode_set() {
        assert_eq!(
            IrcCommand::mode("#channel", Some("+o"), &["nick"]),
            "MODE #channel +o nick\r\n"
        );
    }

    #[test]
    fn test_mode_multiple_params() {
        assert_eq!(
            IrcCommand::mode("#channel", Some("+ov"), &["nick1", "nick2"]),
            "MODE #channel +ov nick1 nick2\r\n"
        );
    }

    #[test]
    fn test_kick_with_reason() {
        assert_eq!(
            IrcCommand::kick("#channel", "baduser", Some("Spamming")),
            "KICK #channel baduser :Spamming\r\n"
        );
    }

    #[test]
    fn test_kick_without_reason() {
        assert_eq!(
            IrcCommand::kick("#channel", "baduser", None),
            "KICK #channel :baduser\r\n"
        );
    }

    #[test]
    fn test_topic_set() {
        assert_eq!(
            IrcCommand::topic("#channel", Some("New topic")),
            "TOPIC #channel :New topic\r\n"
        );
    }

    #[test]
    fn test_topic_query() {
        assert_eq!(IrcCommand::topic("#channel", None), "TOPIC :#channel\r\n");
    }

    #[test]
    fn test_invite() {
        assert_eq!(
            IrcCommand::invite("nick", "#channel"),
            "INVITE nick :#channel\r\n"
        );
    }

    #[test]
    fn test_whois() {
        assert_eq!(IrcCommand::whois("nick"), "WHOIS :nick\r\n");
    }

    #[test]
    fn test_who() {
        assert_eq!(IrcCommand::who("#channel"), "WHO :#channel\r\n");
    }

    #[test]
    fn test_list_with_channel() {
        assert_eq!(IrcCommand::list(Some("#channel")), "LIST :#channel\r\n");
    }

    #[test]
    fn test_list_all() {
        assert_eq!(IrcCommand::list(None), "LIST\r\n");
    }

    #[test]
    fn test_names() {
        assert_eq!(IrcCommand::names("#channel"), "NAMES :#channel\r\n");
    }

    #[test]
    fn test_away_set() {
        assert_eq!(
            IrcCommand::away(Some("Gone to lunch")),
            "AWAY :Gone to lunch\r\n"
        );
    }

    #[test]
    fn test_away_unset() {
        assert_eq!(IrcCommand::away(None), "AWAY\r\n");
    }

    #[test]
    fn test_cap_req() {
        assert_eq!(
            IrcCommand::cap_req(&["multi-prefix", "sasl"]),
            "CAP REQ :multi-prefix sasl\r\n"
        );
    }

    #[test]
    fn test_cap_end() {
        assert_eq!(IrcCommand::cap_end(), "CAP END\r\n");
    }

    #[test]
    fn test_authenticate() {
        assert_eq!(
            IrcCommand::authenticate("PLAIN"),
            "AUTHENTICATE :PLAIN\r\n"
        );
    }

    #[test]
    fn all_commands_end_with_crlf() {
        let commands = vec![
            IrcCommand::pass("p"),
            IrcCommand::nick("n"),
            IrcCommand::user("u", "r"),
            IrcCommand::join("#c"),
            IrcCommand::join_with_key("#c", "k"),
            IrcCommand::part("#c", None),
            IrcCommand::part("#c", Some("bye")),
            IrcCommand::privmsg("#c", "msg"),
            IrcCommand::notice("n", "msg"),
            IrcCommand::quit(None),
            IrcCommand::quit(Some("bye")),
            IrcCommand::ping("t"),
            IrcCommand::pong("t"),
            IrcCommand::mode("#c", None, &[]),
            IrcCommand::mode("#c", Some("+o"), &["n"]),
            IrcCommand::kick("#c", "n", None),
            IrcCommand::kick("#c", "n", Some("reason")),
            IrcCommand::topic("#c", None),
            IrcCommand::topic("#c", Some("t")),
            IrcCommand::invite("n", "#c"),
            IrcCommand::whois("n"),
            IrcCommand::who("m"),
            IrcCommand::list(None),
            IrcCommand::list(Some("#c")),
            IrcCommand::names("#c"),
            IrcCommand::away(None),
            IrcCommand::away(Some("m")),
            IrcCommand::cap_req(&["sasl"]),
            IrcCommand::cap_end(),
            IrcCommand::authenticate("PLAIN"),
        ];
        for cmd in &commands {
            assert!(cmd.ends_with("\r\n"), "Command does not end with CRLF: {cmd}");
        }
    }
}
