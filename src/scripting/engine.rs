use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub expansion: String,
}

#[derive(Debug)]
pub struct ScriptEngine {
    aliases: HashMap<String, Alias>,
    variables: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptResult {
    RawCommand(String),
    Message(String),
    Multiple(Vec<ScriptResult>),
    None,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn add_alias(&mut self, name: &str, expansion: &str) {
        let lower = name.to_ascii_lowercase();
        self.aliases.insert(
            lower,
            Alias {
                name: name.to_string(),
                expansion: expansion.to_string(),
            },
        );
    }

    pub fn remove_alias(&mut self, name: &str) -> bool {
        self.aliases.remove(&name.to_ascii_lowercase()).is_some()
    }

    pub fn list_aliases(&self) -> Vec<&Alias> {
        let mut aliases: Vec<&Alias> = self.aliases.values().collect();
        aliases.sort_by_key(|a| &a.name);
        aliases
    }

    pub fn set_var(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    pub fn process(&self, input: &str, current_target: Option<&str>) -> ScriptResult {
        let input = input.trim();
        if input.is_empty() {
            return ScriptResult::None;
        }

        if input.contains(';') && input.starts_with('/') {
            let parts: Vec<&str> = input.split(';').collect();
            let results: Vec<ScriptResult> = parts
                .iter()
                .map(|p| self.process_single(p.trim(), current_target))
                .filter(|r| !matches!(r, ScriptResult::None))
                .collect();
            if results.is_empty() {
                return ScriptResult::None;
            }
            if results.len() == 1 {
                return results.into_iter().next().unwrap();
            }
            return ScriptResult::Multiple(results);
        }

        self.process_single(input, current_target)
    }

    fn process_single(&self, input: &str, current_target: Option<&str>) -> ScriptResult {
        let input = input.trim();
        if input.is_empty() {
            return ScriptResult::None;
        }

        if let Some(cmd) = input.strip_prefix('/') {
            let (cmd_name, args) = match cmd.find(' ') {
                Some(pos) => (&cmd[..pos], Some(&cmd[pos + 1..])),
                None => (cmd, None),
            };

            let lower = cmd_name.to_ascii_lowercase();
            if let Some(alias) = self.aliases.get(&lower) {
                let mut expanded = alias.expansion.clone();
                if let Some(args) = args {
                    expanded = expanded.replace("$*", args);
                    for (i, arg) in args.split_whitespace().enumerate() {
                        expanded = expanded.replace(&format!("${}", i), arg);
                    }
                } else {
                    expanded = expanded.replace("$*", "");
                }
                if let Some(target) = current_target {
                    expanded = expanded.replace("$target", target);
                }
                let expanded = self.expand_variables(&expanded);
                return self.process(&expanded, current_target);
            }

            let raw = match args {
                Some(a) => format!("{} {}", cmd_name.to_uppercase(), self.expand_variables(a)),
                None => cmd_name.to_uppercase(),
            };
            ScriptResult::RawCommand(raw)
        } else {
            let expanded = self.expand_variables(input);
            ScriptResult::Message(expanded)
        }
    }

    fn expand_variables(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '$'
                && i + 1 < chars.len()
                && (chars[i + 1].is_alphanumeric() || chars[i + 1] == '_')
            {
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                    end += 1;
                }
                let var_name: String = chars[start..end].iter().collect();
                if let Some(val) = self.variables.get(&var_name) {
                    result.push_str(val);
                } else {
                    result.push('$');
                    result.push_str(&var_name);
                }
                i = end;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    pub fn register_defaults(&mut self) {
        self.add_alias("j", "/join $*");
        self.add_alias("l", "/part $*");
        self.add_alias("m", "/msg $*");
        self.add_alias("q", "/quit $*");
        self.add_alias("k", "/kick $target $*");
        self.add_alias("n", "/nick $*");
        self.add_alias("w", "/whois $*");
        self.add_alias("wii", "/whois $0 $0");
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_retrieve_alias() {
        let mut engine = ScriptEngine::new();
        engine.add_alias("j", "/join $*");
        let aliases = engine.list_aliases();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "j");
        assert_eq!(aliases[0].expansion, "/join $*");
    }

    #[test]
    fn alias_lookup_is_case_insensitive() {
        let mut engine = ScriptEngine::new();
        engine.add_alias("Hello", "/say hi");
        let result = engine.process("/hello", None);
        assert_eq!(result, ScriptResult::RawCommand("SAY hi".into()));
    }

    #[test]
    fn remove_alias() {
        let mut engine = ScriptEngine::new();
        engine.add_alias("j", "/join $*");
        assert!(engine.remove_alias("j"));
        assert!(engine.list_aliases().is_empty());
    }

    #[test]
    fn remove_nonexistent_alias() {
        let mut engine = ScriptEngine::new();
        assert!(!engine.remove_alias("nope"));
    }

    #[test]
    fn variable_expansion() {
        let mut engine = ScriptEngine::new();
        engine.set_var("nick", "BitchY");
        engine.set_var("channel", "#rust");
        let result = engine.process("Hello $nick in $channel", None);
        assert_eq!(
            result,
            ScriptResult::Message("Hello BitchY in #rust".into())
        );
    }

    #[test]
    fn get_variable() {
        let mut engine = ScriptEngine::new();
        engine.set_var("foo", "bar");
        assert_eq!(engine.get_var("foo"), Some("bar"));
        assert_eq!(engine.get_var("missing"), None);
    }

    #[test]
    fn unknown_variable_preserved() {
        let engine = ScriptEngine::new();
        let result = engine.process("Hello $unknown", None);
        assert_eq!(result, ScriptResult::Message("Hello $unknown".into()));
    }

    #[test]
    fn process_alias_expansion() {
        let mut engine = ScriptEngine::new();
        engine.add_alias("j", "/join $*");
        let result = engine.process("/j #rust", None);
        assert_eq!(result, ScriptResult::RawCommand("JOIN #rust".into()));
    }

    #[test]
    fn process_alias_with_target() {
        let mut engine = ScriptEngine::new();
        engine.add_alias("k", "/kick $target $*");
        let result = engine.process("/k baduser", Some("#channel"));
        assert_eq!(
            result,
            ScriptResult::RawCommand("KICK #channel baduser".into())
        );
    }

    #[test]
    fn process_regular_message() {
        let engine = ScriptEngine::new();
        let result = engine.process("hello world", None);
        assert_eq!(result, ScriptResult::Message("hello world".into()));
    }

    #[test]
    fn process_raw_command() {
        let engine = ScriptEngine::new();
        let result = engine.process("/join #rust", None);
        assert_eq!(result, ScriptResult::RawCommand("JOIN #rust".into()));
    }

    #[test]
    fn process_command_no_args() {
        let engine = ScriptEngine::new();
        let result = engine.process("/quit", None);
        assert_eq!(result, ScriptResult::RawCommand("QUIT".into()));
    }

    #[test]
    fn process_empty_input() {
        let engine = ScriptEngine::new();
        let result = engine.process("", None);
        assert_eq!(result, ScriptResult::None);
    }

    #[test]
    fn multiple_command_expansion_via_semicolons() {
        let engine = ScriptEngine::new();
        let result = engine.process("/join #a; /join #b", None);
        assert_eq!(
            result,
            ScriptResult::Multiple(vec![
                ScriptResult::RawCommand("JOIN #a".into()),
                ScriptResult::RawCommand("JOIN #b".into()),
            ])
        );
    }

    #[test]
    fn nested_variable_expansion_in_alias() {
        let mut engine = ScriptEngine::new();
        engine.set_var("chan", "#rust");
        engine.add_alias("jc", "/join $chan");
        let result = engine.process("/jc", None);
        assert_eq!(result, ScriptResult::RawCommand("JOIN #rust".into()));
    }

    #[test]
    fn register_defaults_creates_aliases() {
        let mut engine = ScriptEngine::new();
        engine.register_defaults();
        let aliases = engine.list_aliases();
        assert!(aliases.len() >= 7);
        let names: Vec<&str> = aliases.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"j"));
        assert!(names.contains(&"l"));
        assert!(names.contains(&"m"));
        assert!(names.contains(&"q"));
    }

    #[test]
    fn positional_arg_expansion() {
        let mut engine = ScriptEngine::new();
        engine.add_alias("wii", "/whois $0 $0");
        let result = engine.process("/wii someone", None);
        assert_eq!(
            result,
            ScriptResult::RawCommand("WHOIS someone someone".into())
        );
    }

    #[test]
    fn default_trait() {
        let engine = ScriptEngine::default();
        assert!(engine.list_aliases().is_empty());
        assert_eq!(engine.get_var("anything"), None);
    }

    #[test]
    fn variable_set_overwrite() {
        let mut engine = ScriptEngine::new();
        engine.set_var("key", "val1");
        assert_eq!(engine.get_var("key"), Some("val1"));
        engine.set_var("key", "val2");
        assert_eq!(engine.get_var("key"), Some("val2"));
    }
}
