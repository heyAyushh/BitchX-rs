use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct InputState {
    pub buffer: String,
    pub cursor: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub tab_completions: Vec<String>,
    pub tab_index: Option<usize>,
    saved_input: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    SendMessage(String),
    Command(String, Vec<String>),
    TabComplete,
    ScrollUp,
    ScrollDown,
    PreviousChannel,
    NextChannel,
    None,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
            tab_completions: Vec::new(),
            tab_index: None,
            saved_input: String::new(),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.reset_tab();
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            let prev = self.prev_char_boundary();
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
            self.reset_tab();
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.next_char_boundary();
            self.buffer.drain(self.cursor..next);
            self.reset_tab();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = self.next_char_boundary();
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    pub fn delete_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut pos = self.cursor;
        // Skip trailing whitespace
        while pos > 0 && self.buffer.as_bytes().get(pos - 1) == Some(&b' ') {
            pos -= 1;
        }
        // Delete word characters
        while pos > 0 && self.buffer.as_bytes().get(pos - 1) != Some(&b' ') {
            pos -= 1;
        }
        self.buffer.drain(pos..self.cursor);
        self.cursor = pos;
        self.reset_tab();
    }

    pub fn delete_to_end(&mut self) {
        self.buffer.truncate(self.cursor);
        self.reset_tab();
    }

    pub fn delete_to_start(&mut self) {
        self.buffer.drain(..self.cursor);
        self.cursor = 0;
        self.reset_tab();
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.history_index {
            None => {
                self.saved_input = self.buffer.clone();
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => return,
            Some(i) => {
                self.history_index = Some(i - 1);
            }
        }
        if let Some(idx) = self.history_index {
            self.buffer = self.history[idx].clone();
            self.cursor = self.buffer.len();
        }
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            None => {}
            Some(i) => {
                if i + 1 < self.history.len() {
                    self.history_index = Some(i + 1);
                    self.buffer = self.history[i + 1].clone();
                    self.cursor = self.buffer.len();
                } else {
                    self.history_index = None;
                    self.buffer = self.saved_input.clone();
                    self.cursor = self.buffer.len();
                }
            }
        }
    }

    pub fn submit(&mut self) -> InputAction {
        let text = self.buffer.trim().to_string();
        if text.is_empty() {
            return InputAction::None;
        }

        self.history.push(text.clone());
        self.history_index = None;
        self.saved_input.clear();
        self.buffer.clear();
        self.cursor = 0;
        self.reset_tab();

        if let Some(cmd_text) = text.strip_prefix('/') {
            let mut parts = cmd_text.splitn(2, ' ');
            let command = parts.next().unwrap_or("").to_uppercase();
            let args: Vec<String> = parts
                .next()
                .map(|a| a.split_whitespace().map(String::from).collect())
                .unwrap_or_default();
            InputAction::Command(command, args)
        } else {
            InputAction::SendMessage(text)
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputAction {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => InputAction::None,
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                self.move_home();
                InputAction::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.move_end();
                InputAction::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                self.delete_word();
                InputAction::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.delete_to_end();
                InputAction::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.delete_to_start();
                InputAction::None
            }
            (_, KeyCode::Enter) => self.submit(),
            (_, KeyCode::Backspace) => {
                self.delete_char();
                InputAction::None
            }
            (_, KeyCode::Delete) => {
                self.delete_forward();
                InputAction::None
            }
            (KeyModifiers::ALT, KeyCode::Left) => InputAction::PreviousChannel,
            (KeyModifiers::ALT, KeyCode::Right) => InputAction::NextChannel,
            (_, KeyCode::Left) => {
                self.move_left();
                InputAction::None
            }
            (_, KeyCode::Right) => {
                self.move_right();
                InputAction::None
            }
            (_, KeyCode::Home) => {
                self.move_home();
                InputAction::None
            }
            (_, KeyCode::End) => {
                self.move_end();
                InputAction::None
            }
            (_, KeyCode::Up) => {
                self.history_up();
                InputAction::None
            }
            (_, KeyCode::Down) => {
                self.history_down();
                InputAction::None
            }
            (_, KeyCode::Tab) => {
                self.next_completion();
                InputAction::TabComplete
            }
            (_, KeyCode::PageUp) => InputAction::ScrollUp,
            (_, KeyCode::PageDown) => InputAction::ScrollDown,
            (_, KeyCode::Char(c)) => {
                self.insert_char(c);
                InputAction::None
            }
            _ => InputAction::None,
        }
    }

    pub fn set_tab_completions(&mut self, completions: Vec<String>) {
        self.tab_completions = completions;
        self.tab_index = None;
    }

    pub fn next_completion(&mut self) {
        if self.tab_completions.is_empty() {
            return;
        }

        let idx = match self.tab_index {
            None => 0,
            Some(i) => (i + 1) % self.tab_completions.len(),
        };
        self.tab_index = Some(idx);

        // Find the word being completed (last word before cursor)
        let before_cursor = &self.buffer[..self.cursor];
        let word_start = before_cursor.rfind(' ').map(|p| p + 1).unwrap_or(0);

        let completion = &self.tab_completions[idx];
        let after_cursor = self.buffer[self.cursor..].to_string();
        self.buffer.truncate(word_start);
        self.buffer.push_str(completion);
        self.cursor = self.buffer.len();
        self.buffer.push_str(&after_cursor);
    }

    fn prev_char_boundary(&self) -> usize {
        let mut pos = self.cursor.saturating_sub(1);
        while pos > 0 && !self.buffer.is_char_boundary(pos) {
            pos -= 1;
        }
        pos
    }

    fn next_char_boundary(&self) -> usize {
        let mut pos = self.cursor + 1;
        while pos < self.buffer.len() && !self.buffer.is_char_boundary(pos) {
            pos += 1;
        }
        pos.min(self.buffer.len())
    }

    fn reset_tab(&mut self) {
        self.tab_index = None;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn insert_characters() {
        let mut input = InputState::new();
        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.buffer, "hi");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn insert_at_cursor_position() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('c');
        input.move_left();
        input.insert_char('b');
        assert_eq!(input.buffer, "abc");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn delete_char_backspace() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        input.delete_char();
        assert_eq!(input.buffer, "ab");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn delete_char_at_start() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.move_home();
        input.delete_char();
        assert_eq!(input.buffer, "a");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn delete_forward() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('b');
        input.move_home();
        input.delete_forward();
        assert_eq!(input.buffer, "b");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn delete_forward_at_end() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.delete_forward();
        assert_eq!(input.buffer, "a");
    }

    #[test]
    fn cursor_movement() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        assert_eq!(input.cursor, 3);
        input.move_left();
        assert_eq!(input.cursor, 2);
        input.move_left();
        assert_eq!(input.cursor, 1);
        input.move_right();
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn cursor_bounds() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.move_left();
        input.move_left(); // should not go below 0
        assert_eq!(input.cursor, 0);
        input.move_end();
        input.move_right(); // should not go past len
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn home_and_end() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        input.move_home();
        assert_eq!(input.cursor, 0);
        input.move_end();
        assert_eq!(input.cursor, 3);
    }

    #[test]
    fn delete_word() {
        let mut input = InputState::new();
        for c in "hello world".chars() {
            input.insert_char(c);
        }
        input.delete_word();
        assert_eq!(input.buffer, "hello ");
        input.delete_word();
        assert_eq!(input.buffer, "");
    }

    #[test]
    fn delete_word_at_start() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.move_home();
        input.delete_word();
        assert_eq!(input.buffer, "a");
    }

    #[test]
    fn delete_to_end() {
        let mut input = InputState::new();
        for c in "hello world".chars() {
            input.insert_char(c);
        }
        input.cursor = 5;
        input.delete_to_end();
        assert_eq!(input.buffer, "hello");
    }

    #[test]
    fn delete_to_start() {
        let mut input = InputState::new();
        for c in "hello world".chars() {
            input.insert_char(c);
        }
        input.cursor = 6;
        input.delete_to_start();
        assert_eq!(input.buffer, "world");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn submit_message() {
        let mut input = InputState::new();
        for c in "hello".chars() {
            input.insert_char(c);
        }
        let action = input.submit();
        assert_eq!(action, InputAction::SendMessage("hello".to_string()));
        assert!(input.buffer.is_empty());
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn submit_empty() {
        let mut input = InputState::new();
        let action = input.submit();
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn submit_command() {
        let mut input = InputState::new();
        for c in "/join #test".chars() {
            input.insert_char(c);
        }
        let action = input.submit();
        assert_eq!(
            action,
            InputAction::Command("JOIN".to_string(), vec!["#test".to_string()])
        );
    }

    #[test]
    fn submit_command_no_args() {
        let mut input = InputState::new();
        for c in "/quit".chars() {
            input.insert_char(c);
        }
        let action = input.submit();
        assert_eq!(action, InputAction::Command("QUIT".to_string(), vec![]));
    }

    #[test]
    fn submit_adds_to_history() {
        let mut input = InputState::new();
        for c in "first".chars() {
            input.insert_char(c);
        }
        input.submit();
        for c in "second".chars() {
            input.insert_char(c);
        }
        input.submit();
        assert_eq!(input.history.len(), 2);
        assert_eq!(input.history[0], "first");
        assert_eq!(input.history[1], "second");
    }

    #[test]
    fn history_navigation() {
        let mut input = InputState::new();
        for c in "first".chars() {
            input.insert_char(c);
        }
        input.submit();
        for c in "second".chars() {
            input.insert_char(c);
        }
        input.submit();

        // Type something new
        for c in "current".chars() {
            input.insert_char(c);
        }

        input.history_up();
        assert_eq!(input.buffer, "second");
        input.history_up();
        assert_eq!(input.buffer, "first");
        input.history_up(); // at top, should stay
        assert_eq!(input.buffer, "first");
        input.history_down();
        assert_eq!(input.buffer, "second");
        input.history_down();
        assert_eq!(input.buffer, "current"); // back to saved input
    }

    #[test]
    fn history_empty() {
        let mut input = InputState::new();
        input.history_up();
        assert!(input.buffer.is_empty());
        input.history_down();
        assert!(input.buffer.is_empty());
    }

    #[test]
    fn handle_key_chars() {
        let mut input = InputState::new();
        input.handle_key(key(KeyCode::Char('a')));
        input.handle_key(key(KeyCode::Char('b')));
        assert_eq!(input.buffer, "ab");
    }

    #[test]
    fn handle_key_enter() {
        let mut input = InputState::new();
        input.handle_key(key(KeyCode::Char('h')));
        input.handle_key(key(KeyCode::Char('i')));
        let action = input.handle_key(key(KeyCode::Enter));
        assert_eq!(action, InputAction::SendMessage("hi".to_string()));
    }

    #[test]
    fn handle_key_backspace() {
        let mut input = InputState::new();
        input.handle_key(key(KeyCode::Char('a')));
        input.handle_key(key(KeyCode::Char('b')));
        input.handle_key(key(KeyCode::Backspace));
        assert_eq!(input.buffer, "a");
    }

    #[test]
    fn handle_key_ctrl_a_e() {
        let mut input = InputState::new();
        input.handle_key(key(KeyCode::Char('a')));
        input.handle_key(key(KeyCode::Char('b')));
        input.handle_key(ctrl_key('a'));
        assert_eq!(input.cursor, 0);
        input.handle_key(ctrl_key('e'));
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn handle_key_ctrl_w() {
        let mut input = InputState::new();
        for c in "hello world".chars() {
            input.handle_key(key(KeyCode::Char(c)));
        }
        input.handle_key(ctrl_key('w'));
        assert_eq!(input.buffer, "hello ");
    }

    #[test]
    fn handle_key_ctrl_k() {
        let mut input = InputState::new();
        for c in "hello".chars() {
            input.handle_key(key(KeyCode::Char(c)));
        }
        input.handle_key(ctrl_key('a'));
        input.handle_key(ctrl_key('k'));
        assert_eq!(input.buffer, "");
    }

    #[test]
    fn handle_key_ctrl_u() {
        let mut input = InputState::new();
        for c in "hello".chars() {
            input.handle_key(key(KeyCode::Char(c)));
        }
        input.handle_key(ctrl_key('u'));
        assert_eq!(input.buffer, "");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn handle_key_pageup_down() {
        let mut input = InputState::new();
        assert_eq!(
            input.handle_key(key(KeyCode::PageUp)),
            InputAction::ScrollUp
        );
        assert_eq!(
            input.handle_key(key(KeyCode::PageDown)),
            InputAction::ScrollDown
        );
    }

    #[test]
    fn tab_completions() {
        let mut input = InputState::new();
        for c in "he".chars() {
            input.insert_char(c);
        }
        input.set_tab_completions(vec!["hello".to_string(), "help".to_string()]);
        input.next_completion();
        assert_eq!(input.buffer, "hello");
        input.next_completion();
        assert_eq!(input.buffer, "help");
        input.next_completion();
        assert_eq!(input.buffer, "hello"); // wraps
    }

    #[test]
    fn tab_completions_empty() {
        let mut input = InputState::new();
        input.next_completion();
        assert!(input.buffer.is_empty());
    }

    #[test]
    fn unicode_handling() {
        let mut input = InputState::new();
        input.insert_char('é');
        input.insert_char('ñ');
        assert_eq!(input.buffer, "éñ");
        input.delete_char();
        assert_eq!(input.buffer, "é");
    }
}
