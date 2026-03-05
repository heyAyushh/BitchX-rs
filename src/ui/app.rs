use std::collections::HashMap;

use chrono::Utc;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::prelude::*;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::irc::channel::{Channel, ChannelUser, ChatMessage, MessageKind};
use crate::irc::client::{ClientCommand, IrcEvent};
use crate::irc::message::IrcMessage;
use crate::plugin::loader::PluginManager;

use super::input::{InputAction, InputState};
use super::widgets::chat::ChatWidget;
use super::widgets::input_bar::InputBarWidget;
use super::widgets::nick_list::NickListWidget;
use super::widgets::status_bar::StatusBarWidget;

pub struct App {
    pub channels: HashMap<String, Channel>,
    pub active_channel: Option<String>,
    pub server_messages: Vec<ChatMessage>,
    pub input: InputState,
    pub nick: String,
    pub server: String,
    pub user_modes: String,
    pub is_away: bool,
    pub scroll_offset: usize,
    pub running: bool,
    pub lag_ms: Option<u64>,
    pub config: Config,
    pub plugin_manager: PluginManager,

    event_rx: mpsc::UnboundedReceiver<IrcEvent>,
    cmd_tx: mpsc::UnboundedSender<ClientCommand>,
}

impl App {
    pub fn new(
        config: Config,
        event_rx: mpsc::UnboundedReceiver<IrcEvent>,
        cmd_tx: mpsc::UnboundedSender<ClientCommand>,
    ) -> Self {
        let nick = config.nick.clone();
        let server = config
            .servers
            .first()
            .map(|s| s.host.clone())
            .unwrap_or_else(|| "localhost".into());

        let plugin_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".bitchx")
            .join("plugins");

        Self {
            channels: HashMap::new(),
            active_channel: None,
            server_messages: Vec::new(),
            input: InputState::new(),
            nick,
            server,
            user_modes: String::new(),
            is_away: false,
            scroll_offset: 0,
            running: true,
            lag_ms: None,
            config,
            plugin_manager: PluginManager::new(plugin_dir),
            event_rx,
            cmd_tx,
        }
    }

    pub fn handle_irc_event(&mut self, event: IrcEvent) {
        match event {
            IrcEvent::Connected => {
                self.add_server_message("Connected to server", MessageKind::System);
            }
            IrcEvent::Disconnected(reason) => {
                self.add_server_message(&format!("Disconnected: {}", reason), MessageKind::Error);
            }
            IrcEvent::Error(err) => {
                self.add_server_message(&format!("Error: {}", err), MessageKind::Error);
            }
            IrcEvent::LagUpdate(ms) => {
                self.lag_ms = Some(ms);
            }
            IrcEvent::Message(msg) => {
                self.handle_irc_message(msg);
            }
        }
    }

    fn handle_irc_message(&mut self, msg: IrcMessage) {
        let nick = msg.nick().unwrap_or("*").to_string();

        match msg.command.as_str() {
            "PRIVMSG" => {
                if let (Some(target), Some(text)) = (msg.params.first(), msg.trailing()) {
                    let (kind, content) = if text.starts_with('\x01') && text.ends_with('\x01') {
                        let inner = &text[1..text.len() - 1];
                        if let Some(action_text) = inner.strip_prefix("ACTION ") {
                            (MessageKind::Action, action_text.to_string())
                        } else {
                            (MessageKind::Normal, text.to_string())
                        }
                    } else {
                        (MessageKind::Normal, text.to_string())
                    };
                    let msg_sender = nick.clone();
                    let msg_content = content.clone();
                    let chat_msg = ChatMessage {
                        timestamp: Utc::now(),
                        sender: nick,
                        content,
                        kind,
                    };
                    let channel_name = target.to_string();
                    if target.starts_with('#') || target.starts_with('&') {
                        self.ensure_channel(&channel_name);
                        if let Some(ch) = self.channels.get_mut(&channel_name) {
                            ch.add_message(chat_msg);
                            if self.active_channel.as_deref() != Some(target.as_str()) {
                                ch.unread_count += 1;
                            }
                        }
                    } else {
                        self.add_server_message(
                            &format!("<{}> {}", chat_msg.sender, chat_msg.content),
                            MessageKind::Normal,
                        );
                    }

                    let plugin_responses = self.plugin_manager.dispatch_message(
                        &msg_sender,
                        &channel_name,
                        &msg_content,
                    );
                    for (_plugin_name, response) in plugin_responses {
                        let _ = self
                            .cmd_tx
                            .send(ClientCommand::Privmsg(channel_name.clone(), response));
                    }
                }
            }
            "JOIN" => {
                let channel_name = msg
                    .params
                    .first()
                    .or(msg.trailing().map(|_| &msg.params[0]))
                    .cloned()
                    .unwrap_or_default();
                if !channel_name.is_empty() {
                    self.ensure_channel(&channel_name);
                    if nick == self.nick {
                        self.active_channel = Some(channel_name.clone());
                    }
                    if let Some(ch) = self.channels.get_mut(&channel_name) {
                        ch.users.insert(
                            nick.clone(),
                            ChannelUser {
                                nick: nick.clone(),
                                prefix: None,
                            },
                        );
                        ch.add_message(ChatMessage {
                            timestamp: Utc::now(),
                            sender: nick,
                            content: channel_name,
                            kind: MessageKind::Join,
                        });
                    }
                }
            }
            "PART" => {
                if let Some(channel_name) = msg.params.first() {
                    let reason = msg.trailing().unwrap_or("").to_string();
                    if let Some(ch) = self.channels.get_mut(channel_name) {
                        ch.users.remove(&nick);
                        ch.add_message(ChatMessage {
                            timestamp: Utc::now(),
                            sender: nick.clone(),
                            content: format!("{} ({})", channel_name, reason),
                            kind: MessageKind::Part,
                        });
                    }
                    if nick == self.nick {
                        self.channels.remove(channel_name);
                        if self.active_channel.as_deref() == Some(channel_name.as_str()) {
                            self.active_channel = self.channels.keys().next().cloned();
                        }
                    }
                }
            }
            "QUIT" => {
                let reason = msg.trailing().unwrap_or("").to_string();
                let quit_msg = ChatMessage {
                    timestamp: Utc::now(),
                    sender: nick.clone(),
                    content: reason,
                    kind: MessageKind::Quit,
                };
                for ch in self.channels.values_mut() {
                    if ch.users.remove(&nick).is_some() {
                        ch.add_message(quit_msg.clone());
                    }
                }
            }
            "NICK" => {
                let new_nick = msg
                    .trailing()
                    .or(msg.params.first().map(|s| s.as_str()))
                    .unwrap_or(&nick)
                    .to_string();
                if nick == self.nick {
                    self.nick = new_nick.clone();
                }
                let nick_msg = ChatMessage {
                    timestamp: Utc::now(),
                    sender: nick.clone(),
                    content: new_nick.clone(),
                    kind: MessageKind::Nick,
                };
                for ch in self.channels.values_mut() {
                    if let Some(mut user) = ch.users.remove(&nick) {
                        user.nick = new_nick.clone();
                        ch.users.insert(new_nick.clone(), user);
                        ch.add_message(nick_msg.clone());
                    }
                }
            }
            "KICK" => {
                if let (Some(channel_name), Some(kicked)) = (msg.params.first(), msg.params.get(1))
                {
                    let reason = msg.trailing().unwrap_or("").to_string();
                    if let Some(ch) = self.channels.get_mut(channel_name) {
                        ch.users.remove(kicked);
                        ch.add_message(ChatMessage {
                            timestamp: Utc::now(),
                            sender: kicked.clone(),
                            content: format!("by {} ({})", nick, reason),
                            kind: MessageKind::Kick,
                        });
                    }
                    if kicked == &self.nick {
                        self.channels.remove(channel_name);
                        if self.active_channel.as_deref() == Some(channel_name.as_str()) {
                            self.active_channel = self.channels.keys().next().cloned();
                        }
                    }
                }
            }
            "MODE" => {
                if let Some(target) = msg.params.first() {
                    let mode_str = msg.params[1..].join(" ");
                    if target.starts_with('#') || target.starts_with('&') {
                        if let Some(ch) = self.channels.get_mut(target) {
                            ch.add_message(ChatMessage {
                                timestamp: Utc::now(),
                                sender: nick,
                                content: mode_str,
                                kind: MessageKind::Mode,
                            });
                        }
                    } else if target == &self.nick {
                        self.user_modes = mode_str;
                    }
                }
            }
            "TOPIC" => {
                if let (Some(channel_name), Some(topic_text)) = (msg.params.first(), msg.trailing())
                {
                    if let Some(ch) = self.channels.get_mut(channel_name) {
                        ch.topic = Some(topic_text.to_string());
                        ch.add_message(ChatMessage {
                            timestamp: Utc::now(),
                            sender: nick,
                            content: topic_text.to_string(),
                            kind: MessageKind::Topic,
                        });
                    }
                }
            }
            "NOTICE" => {
                if let Some(text) = msg.trailing() {
                    self.add_server_message(&format!("-{}- {}", nick, text), MessageKind::Notice);
                }
            }
            // 353 = RPL_NAMREPLY
            "353" => {
                if let (Some(channel_name), Some(names)) = (msg.params.get(2), msg.trailing()) {
                    self.ensure_channel(channel_name);
                    if let Some(ch) = self.channels.get_mut(channel_name) {
                        for name in names.split_whitespace() {
                            let (prefix, nick_str) =
                                if name.starts_with('@') || name.starts_with('+') {
                                    (Some(name.as_bytes()[0] as char), &name[1..])
                                } else {
                                    (None, name)
                                };
                            ch.users.insert(
                                nick_str.to_string(),
                                ChannelUser {
                                    nick: nick_str.to_string(),
                                    prefix,
                                },
                            );
                        }
                    }
                }
            }
            // 332 = RPL_TOPIC
            "332" => {
                if let (Some(channel_name), Some(topic_text)) = (msg.params.get(1), msg.trailing())
                {
                    self.ensure_channel(channel_name);
                    if let Some(ch) = self.channels.get_mut(channel_name) {
                        ch.topic = Some(topic_text.to_string());
                    }
                }
            }
            _ => {
                if let Some(text) = msg.trailing() {
                    self.add_server_message(text, MessageKind::System);
                }
            }
        }
    }

    pub fn handle_input_action(&mut self, action: InputAction) {
        match action {
            InputAction::SendMessage(text) => {
                if let Some(ref target) = self.active_channel {
                    let _ = self
                        .cmd_tx
                        .send(ClientCommand::Privmsg(target.clone(), text.clone()));
                    let msg = ChatMessage {
                        timestamp: Utc::now(),
                        sender: self.nick.clone(),
                        content: text,
                        kind: MessageKind::Normal,
                    };
                    if let Some(ch) = self.channels.get_mut(target) {
                        ch.add_message(msg);
                    }
                }
                self.scroll_offset = 0;
            }
            InputAction::Command(cmd, args) => {
                self.handle_command(&cmd, &args);
                self.scroll_offset = 0;
            }
            InputAction::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_add(5);
            }
            InputAction::ScrollDown => {
                self.scroll_offset = self.scroll_offset.saturating_sub(5);
            }
            InputAction::PreviousChannel => {
                self.switch_channel(-1);
            }
            InputAction::NextChannel => {
                self.switch_channel(1);
            }
            InputAction::TabComplete => {
                if let Some(ref ch_name) = self.active_channel.clone() {
                    if let Some(ch) = self.channels.get(ch_name) {
                        let partial = self.current_word();
                        let completions: Vec<String> = ch
                            .users
                            .values()
                            .filter(|u| u.nick.to_lowercase().starts_with(&partial.to_lowercase()))
                            .map(|u| u.nick.clone())
                            .collect();
                        self.input.set_tab_completions(completions);
                    }
                }
            }
            InputAction::None => {}
        }
    }

    fn handle_command(&mut self, cmd: &str, args: &[String]) {
        match cmd {
            "JOIN" | "J" => {
                if let Some(channel) = args.first() {
                    let _ = self.cmd_tx.send(ClientCommand::Join(channel.clone()));
                }
            }
            "PART" | "LEAVE" => {
                let channel = args
                    .first()
                    .cloned()
                    .or_else(|| self.active_channel.clone());
                if let Some(ch) = channel {
                    let reason = if args.len() > 1 {
                        Some(args[1..].join(" "))
                    } else {
                        None
                    };
                    let _ = self.cmd_tx.send(ClientCommand::Part(ch, reason));
                }
            }
            "MSG" | "PRIVMSG" => {
                if args.len() >= 2 {
                    let target = args[0].clone();
                    let text = args[1..].join(" ");
                    let _ = self.cmd_tx.send(ClientCommand::Privmsg(target, text));
                }
            }
            "NICK" => {
                if let Some(new_nick) = args.first() {
                    let _ = self.cmd_tx.send(ClientCommand::Nick(new_nick.clone()));
                }
            }
            "QUIT" | "EXIT" => {
                let reason = if args.is_empty() {
                    None
                } else {
                    Some(args.join(" "))
                };
                let _ = self.cmd_tx.send(ClientCommand::Quit(reason));
                self.running = false;
            }
            "TOPIC" => {
                if let Some(ref ch_name) = self.active_channel.clone() {
                    let topic = if args.is_empty() {
                        None
                    } else {
                        Some(args.join(" "))
                    };
                    let _ = self
                        .cmd_tx
                        .send(ClientCommand::Topic(ch_name.clone(), topic));
                }
            }
            "AWAY" => {
                let reason = if args.is_empty() {
                    None
                } else {
                    Some(args.join(" "))
                };
                self.is_away = reason.is_some();
                let _ = self.cmd_tx.send(ClientCommand::Away(reason));
            }
            "MODE" => {
                if args.len() >= 2 {
                    let target = args[0].clone();
                    let mode_str = args[1..].join(" ");
                    let _ = self.cmd_tx.send(ClientCommand::Mode(target, mode_str));
                }
            }
            "KICK" => {
                if let Some(ref ch_name) = self.active_channel.clone() {
                    if let Some(target_nick) = args.first() {
                        let reason = if args.len() > 1 {
                            Some(args[1..].join(" "))
                        } else {
                            None
                        };
                        let _ = self.cmd_tx.send(ClientCommand::Kick(
                            ch_name.clone(),
                            target_nick.clone(),
                            reason,
                        ));
                    }
                }
            }
            "LOADDLL" => {
                if let Some(path_str) = args.first() {
                    let path = std::path::Path::new(path_str);
                    match self.plugin_manager.load(path) {
                        Ok(name) => {
                            self.add_server_message(
                                &format!("Plugin '{}' loaded successfully", name),
                                MessageKind::System,
                            );
                        }
                        Err(e) => {
                            self.add_server_message(
                                &format!("Failed to load plugin: {}", e),
                                MessageKind::Error,
                            );
                        }
                    }
                } else {
                    self.add_server_message(
                        "Usage: /loaddll <path>",
                        MessageKind::Error,
                    );
                }
            }
            "UNLOADDLL" => {
                if let Some(name) = args.first() {
                    match self.plugin_manager.unload(name) {
                        Ok(()) => {
                            self.add_server_message(
                                &format!("Plugin '{}' unloaded successfully", name),
                                MessageKind::System,
                            );
                        }
                        Err(e) => {
                            self.add_server_message(
                                &format!("Failed to unload plugin: {}", e),
                                MessageKind::Error,
                            );
                        }
                    }
                } else {
                    self.add_server_message(
                        "Usage: /unloaddll <name>",
                        MessageKind::Error,
                    );
                }
            }
            "LISTDLL" => {
                let plugins: Vec<(String, String, String, String)> = self
                    .plugin_manager
                    .list()
                    .into_iter()
                    .map(|(n, v, d, p)| {
                        (
                            n.to_string(),
                            v.to_string(),
                            d.to_string(),
                            p.display().to_string(),
                        )
                    })
                    .collect();
                if plugins.is_empty() {
                    self.add_server_message("No plugins loaded", MessageKind::System);
                } else {
                    self.add_server_message(
                        &format!("Loaded plugins ({}):", plugins.len()),
                        MessageKind::System,
                    );
                    for (name, version, description, path) in &plugins {
                        self.add_server_message(
                            &format!("  {} v{} - {} ({})", name, version, description, path),
                            MessageKind::System,
                        );
                    }
                }
            }
            _ => {
                let raw = if args.is_empty() {
                    cmd.to_string()
                } else {
                    format!("{} {}", cmd, args.join(" "))
                };
                let _ = self.cmd_tx.send(ClientCommand::SendRaw(raw));
            }
        }
    }

    pub fn active_messages(&self) -> &[ChatMessage] {
        if let Some(ref name) = self.active_channel {
            if let Some(ch) = self.channels.get(name) {
                return &ch.messages;
            }
        }
        &self.server_messages
    }

    pub fn channel_list(&self) -> Vec<(String, usize)> {
        let mut list: Vec<(String, usize)> = self
            .channels
            .iter()
            .map(|(name, ch)| (name.clone(), ch.unread_count))
            .collect();
        list.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        list
    }

    pub fn active_users(&self) -> Vec<&ChannelUser> {
        if let Some(ref name) = self.active_channel {
            if let Some(ch) = self.channels.get(name) {
                return ch.sorted_users();
            }
        }
        Vec::new()
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let bg_block = ratatui::widgets::Block::default().style(Style::reset());
        frame.render_widget(bg_block, area);

        // Main vertical layout: chat area, status bar (2 lines), input bar (3 lines)
        let status_height = if self.channels.is_empty() { 1 } else { 2 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(status_height),
                Constraint::Length(3),
            ])
            .split(area);

        let chat_area = chunks[0];
        let status_area = chunks[1];
        let input_area = chunks[2];

        // Horizontal split for chat + nick list
        let show_nick_list = self.config.ui.show_nick_list && self.active_channel.is_some();
        if show_nick_list {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(20),
                    Constraint::Length(self.config.ui.nick_list_width),
                ])
                .split(chat_area);

            self.render_chat(frame, h_chunks[0]);
            self.render_nick_list(frame, h_chunks[1]);
        } else {
            self.render_chat(frame, chat_area);
        }

        self.render_status_bar(frame, status_area);
        self.render_input_bar(frame, input_area);
    }

    fn render_chat(&self, frame: &mut Frame, area: Rect) {
        let messages = self.active_messages();
        let chat = ChatWidget::new(messages)
            .scroll_offset(self.scroll_offset)
            .show_timestamps(self.config.ui.timestamps);
        frame.render_widget(chat, area);
    }

    fn render_nick_list(&self, frame: &mut Frame, area: Rect) {
        let users = self.active_users();
        let user_refs: Vec<&ChannelUser> = users;
        let title = self
            .active_channel
            .as_deref()
            .map(|ch| format!(" {} ({}) ", ch, user_refs.len()))
            .unwrap_or_else(|| " Users ".to_string());
        // We need a Vec<&ChannelUser> for the widget
        let nick_list = NickListWidget::new(&user_refs, &title);
        frame.render_widget(nick_list, area);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let channel_list = self.channel_list();
        let user_count = self
            .active_channel
            .as_ref()
            .and_then(|name| self.channels.get(name))
            .map(|ch| ch.users.len())
            .unwrap_or(0);

        let status = StatusBarWidget {
            nick: &self.nick,
            channel: self.active_channel.as_deref(),
            user_count,
            lag_ms: self.lag_ms,
            is_away: self.is_away,
            server: &self.server,
            modes: &self.user_modes,
            channels: &channel_list,
        };
        frame.render_widget(status, area);
    }

    fn render_input_bar(&self, frame: &mut Frame, area: Rect) {
        let prompt = self
            .active_channel
            .as_ref()
            .map(|ch| format!("[{}] ", ch))
            .unwrap_or_else(|| "> ".to_string());
        let input = InputBarWidget::new(&self.input.buffer, self.input.cursor).prompt(&prompt);
        frame.render_widget(input, area);
    }

    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> anyhow::Result<()> {
        let mut reader = EventStream::new();

        while self.running {
            terminal.draw(|f| self.render(f))?;

            tokio::select! {
                Some(irc_event) = self.event_rx.recv() => {
                    self.handle_irc_event(irc_event);
                }
                maybe_event = reader.next() => {
                    match maybe_event {
                        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                            if key.modifiers.contains(KeyModifiers::CONTROL)
                                && key.code == KeyCode::Char('c')
                            {
                                self.running = false;
                                break;
                            }
                            let action = self.input.handle_key(key);
                            self.handle_input_action(action);
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_channel(&mut self, name: &str) {
        if !self.channels.contains_key(name) {
            let mut ch = Channel::new(name.to_string());
            ch.max_messages = self.config.ui.scrollback_lines;
            self.channels.insert(name.to_string(), ch);
        }
    }

    fn add_server_message(&mut self, content: &str, kind: MessageKind) {
        self.server_messages.push(ChatMessage {
            timestamp: Utc::now(),
            sender: String::new(),
            content: content.to_string(),
            kind,
        });
        let max = self.config.ui.scrollback_lines;
        if max > 0 && self.server_messages.len() > max {
            self.server_messages.drain(..self.server_messages.len() - max);
        }
    }

    fn switch_channel(&mut self, direction: i32) {
        let list = self.channel_list();
        if list.is_empty() {
            return;
        }
        let current_idx = self
            .active_channel
            .as_ref()
            .and_then(|name| list.iter().position(|(n, _)| n == name));
        let new_idx = match current_idx {
            Some(idx) => {
                let len = list.len() as i32;
                ((idx as i32 + direction).rem_euclid(len)) as usize
            }
            None => 0,
        };
        self.active_channel = Some(list[new_idx].0.clone());
        if let Some(ch) = self
            .active_channel
            .as_ref()
            .and_then(|n| self.channels.get_mut(n))
        {
            ch.unread_count = 0;
        }
        self.scroll_offset = 0;
    }

    fn current_word(&self) -> String {
        let before = &self.input.buffer[..self.input.cursor];
        before
            .rsplit_once(' ')
            .map(|(_, w)| w)
            .unwrap_or(before)
            .to_string()
    }
}
