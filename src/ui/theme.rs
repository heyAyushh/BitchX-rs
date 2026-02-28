use ratatui::style::{Color, Modifier, Style};

use crate::irc::channel::MessageKind;

pub struct Theme;

const NICK_COLORS: [Color; 12] = [
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::Blue,
    Color::Red,
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightBlue,
    Color::LightRed,
];

impl Theme {
    pub fn nick_color(nick: &str) -> Color {
        let hash: usize = nick.bytes().fold(0usize, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as usize)
        });
        NICK_COLORS[hash % NICK_COLORS.len()]
    }

    pub fn message_style(kind: &MessageKind) -> Style {
        match kind {
            MessageKind::Normal => Style::default().fg(Color::White),
            MessageKind::Action => Style::default().fg(Color::Magenta),
            MessageKind::Notice => Style::default().fg(Color::Yellow),
            MessageKind::System => Style::default().fg(Color::Cyan),
            MessageKind::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            MessageKind::Join => Style::default().fg(Color::Green),
            MessageKind::Part => Style::default().fg(Color::Yellow),
            MessageKind::Quit => Style::default().fg(Color::DarkGray),
            MessageKind::Kick => Style::default().fg(Color::Red),
            MessageKind::Mode => Style::default().fg(Color::Cyan),
            MessageKind::Topic => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            MessageKind::Nick => Style::default().fg(Color::Cyan),
        }
    }

    pub fn status_bar() -> Style {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn input_bar() -> Style {
        Style::default().fg(Color::White).bg(Color::Black)
    }

    pub fn nick_list_header() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn nick_op() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }

    pub fn nick_voice() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn nick_normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn channel_active() -> Style {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }

    pub fn channel_inactive() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn channel_unread() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn timestamp() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn topic() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray).bg(Color::Black)
    }

    pub fn highlight() -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }
}
