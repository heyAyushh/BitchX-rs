use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct User {
    pub nick: String,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub realname: Option<String>,
    pub server: Option<String>,
    pub away: Option<String>,
    pub is_oper: bool,
    pub channels: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct UserTracker {
    users: HashMap<String, User>,
}

impl User {
    pub fn new(nick: &str) -> Self {
        Self {
            nick: nick.to_string(),
            username: None,
            hostname: None,
            realname: None,
            server: None,
            away: None,
            is_oper: false,
            channels: HashSet::new(),
        }
    }

    pub fn hostmask(&self) -> String {
        let user = self.username.as_deref().unwrap_or("*");
        let host = self.hostname.as_deref().unwrap_or("*");
        format!("{}!{}@{}", self.nick, user, host)
    }

    pub fn is_away(&self) -> bool {
        self.away.is_some()
    }
}

impl UserTracker {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_or_update(&mut self, nick: &str) -> &mut User {
        let key = nick.to_ascii_lowercase();
        self.users
            .entry(key)
            .and_modify(|u| u.nick = nick.to_string())
            .or_insert_with(|| User::new(nick))
    }

    pub fn remove(&mut self, nick: &str) -> Option<User> {
        self.users.remove(&nick.to_ascii_lowercase())
    }

    pub fn rename(&mut self, old_nick: &str, new_nick: &str) {
        let old_key = old_nick.to_ascii_lowercase();
        if let Some(mut user) = self.users.remove(&old_key) {
            user.nick = new_nick.to_string();
            self.users.insert(new_nick.to_ascii_lowercase(), user);
        }
    }

    pub fn get(&self, nick: &str) -> Option<&User> {
        self.users.get(&nick.to_ascii_lowercase())
    }

    pub fn get_mut(&mut self, nick: &str) -> Option<&mut User> {
        self.users.get_mut(&nick.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let u = User::new("Alice");
        assert_eq!(u.nick, "Alice");
        assert!(u.username.is_none());
        assert!(u.hostname.is_none());
        assert!(!u.is_oper);
        assert!(u.channels.is_empty());
    }

    #[test]
    fn test_hostmask_full() {
        let mut u = User::new("Alice");
        u.username = Some("alice".into());
        u.hostname = Some("example.com".into());
        assert_eq!(u.hostmask(), "Alice!alice@example.com");
    }

    #[test]
    fn test_hostmask_partial() {
        let u = User::new("Bob");
        assert_eq!(u.hostmask(), "Bob!*@*");
    }

    #[test]
    fn test_hostmask_username_only() {
        let mut u = User::new("Carol");
        u.username = Some("carol".into());
        assert_eq!(u.hostmask(), "Carol!carol@*");
    }

    #[test]
    fn test_is_away() {
        let mut u = User::new("Dave");
        assert!(!u.is_away());
        u.away = Some("Gone fishing".into());
        assert!(u.is_away());
        u.away = None;
        assert!(!u.is_away());
    }

    #[test]
    fn test_tracker_add_and_get() {
        let mut tracker = UserTracker::new();
        tracker.add_or_update("Alice");
        assert!(tracker.get("Alice").is_some());
        assert!(tracker.get("alice").is_some());
        assert!(tracker.get("ALICE").is_some());
    }

    #[test]
    fn test_tracker_add_or_update_preserves_data() {
        let mut tracker = UserTracker::new();
        {
            let u = tracker.add_or_update("Alice");
            u.username = Some("alice".into());
            u.hostname = Some("host.com".into());
        }
        {
            let u = tracker.add_or_update("alice");
            assert_eq!(u.nick, "alice");
            assert_eq!(u.username.as_deref(), Some("alice"));
            assert_eq!(u.hostname.as_deref(), Some("host.com"));
        }
    }

    #[test]
    fn test_tracker_remove() {
        let mut tracker = UserTracker::new();
        tracker.add_or_update("Alice");
        let removed = tracker.remove("ALICE");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().nick, "Alice");
        assert!(tracker.get("Alice").is_none());
    }

    #[test]
    fn test_tracker_remove_nonexistent() {
        let mut tracker = UserTracker::new();
        assert!(tracker.remove("ghost").is_none());
    }

    #[test]
    fn test_tracker_rename() {
        let mut tracker = UserTracker::new();
        {
            let u = tracker.add_or_update("OldNick");
            u.username = Some("user".into());
        }
        tracker.rename("OldNick", "NewNick");
        assert!(tracker.get("OldNick").is_none());
        let u = tracker.get("NewNick").unwrap();
        assert_eq!(u.nick, "NewNick");
        assert_eq!(u.username.as_deref(), Some("user"));
    }

    #[test]
    fn test_tracker_rename_nonexistent() {
        let mut tracker = UserTracker::new();
        tracker.rename("ghost", "phantom");
        assert!(tracker.get("phantom").is_none());
    }

    #[test]
    fn test_tracker_get_mut() {
        let mut tracker = UserTracker::new();
        tracker.add_or_update("Alice");
        {
            let u = tracker.get_mut("alice").unwrap();
            u.is_oper = true;
            u.away = Some("brb".into());
        }
        let u = tracker.get("Alice").unwrap();
        assert!(u.is_oper);
        assert_eq!(u.away.as_deref(), Some("brb"));
    }

    #[test]
    fn test_tracker_default() {
        let tracker = UserTracker::default();
        assert!(tracker.get("anyone").is_none());
    }

    #[test]
    fn test_user_channels() {
        let mut u = User::new("Alice");
        u.channels.insert("#test".into());
        u.channels.insert("#rust".into());
        assert_eq!(u.channels.len(), 2);
        assert!(u.channels.contains("#test"));
    }
}
