use crate::irc::message::IrcMessage;

#[derive(Debug, Clone)]
pub enum IrcEvent {
    Connected,
    Disconnected(String),
    Message(IrcMessage),
    Error(String),
    LagUpdate(u64),
}

#[derive(Debug, Clone)]
pub enum ClientCommand {
    SendRaw(String),
    Join(String),
    Part(String, Option<String>),
    Privmsg(String, String),
    Notice(String, String),
    Nick(String),
    Quit(Option<String>),
    Mode(String, String),
    Kick(String, String, Option<String>),
    Topic(String, Option<String>),
    Away(Option<String>),
    Ctcp(String, String),
    Ping(String),
}
