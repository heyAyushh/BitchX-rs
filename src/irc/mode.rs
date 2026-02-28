use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum ModeChange {
    Add(char, Option<String>),
    Remove(char, Option<String>),
}

#[derive(Debug, Clone, Default)]
pub struct ChannelModes {
    pub flags: HashSet<char>,
    pub key: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UserModes {
    pub flags: HashSet<char>,
}

/// Modes that take a parameter when set
pub fn mode_takes_param_on_set(mode: char) -> bool {
    matches!(mode, 'o' | 'v' | 'b' | 'k' | 'l' | 'e' | 'I' | 'h' | 'q')
}

/// Modes that take a parameter when unset
pub fn mode_takes_param_on_unset(mode: char) -> bool {
    matches!(mode, 'o' | 'v' | 'b' | 'k' | 'e' | 'I' | 'h' | 'q')
}

/// Parse a mode string like "+ov nick1 nick2" or "+ntl 10"
pub fn parse_mode_changes(mode_str: &str, params: &[String]) -> Vec<ModeChange> {
    let mut changes = Vec::new();
    let mut adding = true;
    let mut param_idx = 0;

    for ch in mode_str.chars() {
        match ch {
            '+' => adding = true,
            '-' => adding = false,
            _ => {
                let needs_param = if adding {
                    mode_takes_param_on_set(ch)
                } else {
                    mode_takes_param_on_unset(ch)
                };

                let param = if needs_param && param_idx < params.len() {
                    let p = params[param_idx].clone();
                    param_idx += 1;
                    Some(p)
                } else {
                    None
                };

                if adding {
                    changes.push(ModeChange::Add(ch, param));
                } else {
                    changes.push(ModeChange::Remove(ch, param));
                }
            }
        }
    }

    changes
}

impl ChannelModes {
    pub fn apply(&mut self, change: &ModeChange) {
        match change {
            ModeChange::Add(mode, param) => match *mode {
                'k' => {
                    self.key = param.clone();
                    self.flags.insert('k');
                }
                'l' => {
                    self.limit = param.as_ref().and_then(|p| p.parse().ok());
                    self.flags.insert('l');
                }
                _ => {
                    self.flags.insert(*mode);
                }
            },
            ModeChange::Remove(mode, _param) => match *mode {
                'k' => {
                    self.key = None;
                    self.flags.remove(&'k');
                }
                'l' => {
                    self.limit = None;
                    self.flags.remove(&'l');
                }
                _ => {
                    self.flags.remove(mode);
                }
            },
        }
    }

    pub fn to_mode_string(&self) -> String {
        if self.flags.is_empty() {
            return String::new();
        }
        let mut modes: Vec<char> = self.flags.iter().copied().collect();
        modes.sort();
        let mut out = "+".to_string();
        out.extend(modes.iter());

        let mut params = Vec::new();
        if let Some(ref key) = self.key {
            params.push(key.clone());
        }
        if let Some(limit) = self.limit {
            params.push(limit.to_string());
        }
        if !params.is_empty() {
            out.push(' ');
            out.push_str(&params.join(" "));
        }
        out
    }
}

impl UserModes {
    pub fn apply(&mut self, change: &ModeChange) {
        match change {
            ModeChange::Add(mode, _) => {
                self.flags.insert(*mode);
            }
            ModeChange::Remove(mode, _) => {
                self.flags.remove(mode);
            }
        }
    }

    pub fn has(&self, mode: char) -> bool {
        self.flags.contains(&mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plus_o_nick() {
        let changes = parse_mode_changes("+o", &["nick".into()]);
        assert_eq!(changes, vec![ModeChange::Add('o', Some("nick".into()))]);
    }

    #[test]
    fn parse_plus_ntl() {
        let changes = parse_mode_changes("+ntl", &["10".into()]);
        assert_eq!(
            changes,
            vec![
                ModeChange::Add('n', None),
                ModeChange::Add('t', None),
                ModeChange::Add('l', Some("10".into())),
            ]
        );
    }

    #[test]
    fn parse_minus_o_plus_v() {
        let changes = parse_mode_changes("-o+v", &["nick1".into(), "nick2".into()]);
        assert_eq!(
            changes,
            vec![
                ModeChange::Remove('o', Some("nick1".into())),
                ModeChange::Add('v', Some("nick2".into())),
            ]
        );
    }

    #[test]
    fn parse_plus_k_secret() {
        let changes = parse_mode_changes("+k", &["secret".into()]);
        assert_eq!(changes, vec![ModeChange::Add('k', Some("secret".into()))]);
    }

    #[test]
    fn parse_minus_k() {
        let changes = parse_mode_changes("-k", &["oldkey".into()]);
        assert_eq!(
            changes,
            vec![ModeChange::Remove('k', Some("oldkey".into()))]
        );
    }

    #[test]
    fn parse_plus_ov_two_nicks() {
        let changes = parse_mode_changes("+ov", &["nick1".into(), "nick2".into()]);
        assert_eq!(
            changes,
            vec![
                ModeChange::Add('o', Some("nick1".into())),
                ModeChange::Add('v', Some("nick2".into())),
            ]
        );
    }

    #[test]
    fn parse_simple_flags_no_params() {
        let changes = parse_mode_changes("+nt", &[]);
        assert_eq!(
            changes,
            vec![ModeChange::Add('n', None), ModeChange::Add('t', None),]
        );
    }

    #[test]
    fn parse_minus_nt() {
        let changes = parse_mode_changes("-nt", &[]);
        assert_eq!(
            changes,
            vec![ModeChange::Remove('n', None), ModeChange::Remove('t', None),]
        );
    }

    #[test]
    fn apply_modes_to_channel() {
        let mut modes = ChannelModes::default();
        let changes = parse_mode_changes("+ntl", &["10".into()]);
        for c in &changes {
            modes.apply(c);
        }
        assert!(modes.flags.contains(&'n'));
        assert!(modes.flags.contains(&'t'));
        assert!(modes.flags.contains(&'l'));
        assert_eq!(modes.limit, Some(10));
    }

    #[test]
    fn apply_key_mode() {
        let mut modes = ChannelModes::default();
        modes.apply(&ModeChange::Add('k', Some("secret".into())));
        assert_eq!(modes.key, Some("secret".into()));
        assert!(modes.flags.contains(&'k'));

        modes.apply(&ModeChange::Remove('k', None));
        assert_eq!(modes.key, None);
        assert!(!modes.flags.contains(&'k'));
    }

    #[test]
    fn apply_limit_mode() {
        let mut modes = ChannelModes::default();
        modes.apply(&ModeChange::Add('l', Some("50".into())));
        assert_eq!(modes.limit, Some(50));

        modes.apply(&ModeChange::Remove('l', None));
        assert_eq!(modes.limit, None);
    }

    #[test]
    fn remove_flag_mode() {
        let mut modes = ChannelModes::default();
        modes.apply(&ModeChange::Add('n', None));
        modes.apply(&ModeChange::Add('t', None));
        assert!(modes.flags.contains(&'n'));

        modes.apply(&ModeChange::Remove('n', None));
        assert!(!modes.flags.contains(&'n'));
        assert!(modes.flags.contains(&'t'));
    }

    #[test]
    fn channel_modes_to_string_empty() {
        let modes = ChannelModes::default();
        assert_eq!(modes.to_mode_string(), "");
    }

    #[test]
    fn channel_modes_to_string_flags() {
        let mut modes = ChannelModes::default();
        modes.flags.insert('n');
        modes.flags.insert('t');
        let s = modes.to_mode_string();
        assert!(s.starts_with('+'));
        assert!(s.contains('n'));
        assert!(s.contains('t'));
    }

    #[test]
    fn user_modes_apply_and_has() {
        let mut modes = UserModes::default();
        assert!(!modes.has('i'));

        modes.apply(&ModeChange::Add('i', None));
        assert!(modes.has('i'));

        modes.apply(&ModeChange::Remove('i', None));
        assert!(!modes.has('i'));
    }

    #[test]
    fn user_modes_multiple() {
        let mut modes = UserModes::default();
        modes.apply(&ModeChange::Add('i', None));
        modes.apply(&ModeChange::Add('w', None));
        assert!(modes.has('i'));
        assert!(modes.has('w'));
        assert!(!modes.has('o'));
    }

    #[test]
    fn mode_takes_param_checks() {
        assert!(mode_takes_param_on_set('o'));
        assert!(mode_takes_param_on_set('v'));
        assert!(mode_takes_param_on_set('k'));
        assert!(mode_takes_param_on_set('l'));
        assert!(mode_takes_param_on_set('b'));
        assert!(!mode_takes_param_on_set('n'));
        assert!(!mode_takes_param_on_set('t'));

        assert!(mode_takes_param_on_unset('o'));
        assert!(mode_takes_param_on_unset('k'));
        assert!(!mode_takes_param_on_unset('l'));
        assert!(!mode_takes_param_on_unset('n'));
    }

    #[test]
    fn parse_ban_mode() {
        let changes = parse_mode_changes("+b", &["*!*@bad.host".into()]);
        assert_eq!(
            changes,
            vec![ModeChange::Add('b', Some("*!*@bad.host".into()))]
        );
    }

    #[test]
    fn parse_complex_mode_string() {
        let changes = parse_mode_changes(
            "+o-v+b",
            &["nick1".into(), "nick2".into(), "*!*@host".into()],
        );
        assert_eq!(
            changes,
            vec![
                ModeChange::Add('o', Some("nick1".into())),
                ModeChange::Remove('v', Some("nick2".into())),
                ModeChange::Add('b', Some("*!*@host".into())),
            ]
        );
    }
}
