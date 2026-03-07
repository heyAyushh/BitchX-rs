use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageKind {
    Normal,
    Action,
    Notice,
    System,
    Error,
    Join,
    Part,
    Quit,
    Kick,
    Mode,
    Topic,
    Nick,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub content: String,
    pub kind: MessageKind,
}

#[derive(Debug, Clone)]
pub struct ChannelUser {
    pub nick: String,
    pub prefix: Option<char>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub name: String,
    pub topic: Option<String>,
    pub users: HashMap<String, ChannelUser>,
    pub modes: HashSet<char>,
    pub messages: Vec<ChatMessage>,
    pub unread_count: usize,
    pub max_messages: usize,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Self {
            name,
            topic: None,
            users: HashMap::new(),
            modes: HashSet::new(),
            messages: Vec::new(),
            unread_count: 0,
            max_messages: 0,
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        if self.max_messages > 0 && self.messages.len() > self.max_messages {
            self.messages
                .drain(..self.messages.len() - self.max_messages);
        }
    }

    pub fn sorted_users(&self) -> Vec<&ChannelUser> {
        let mut users: Vec<&ChannelUser> = self.users.values().collect();
        users.sort_by(|a, b| {
            let rank = |u: &&ChannelUser| match u.prefix {
                Some('@') => 0,
                Some('+') => 1,
                _ => 2,
            };
            rank(a)
                .cmp(&rank(b))
                .then_with(|| a.nick.to_lowercase().cmp(&b.nick.to_lowercase()))
        });
        users
    }
}
