use std::collections::HashMap;

/// Minimal Valve KeyValues (text VDF) parser, enough for `appmanifest_*.acf`
/// and `libraryfolders.vdf`.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(map) => map.get(key),
            Value::String(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value),
            Value::Object(_) => None,
        }
    }

    pub fn entries(&self) -> Vec<(&String, &Value)> {
        match self {
            Value::Object(map) => map.iter().collect(),
            Value::String(_) => Vec::new(),
        }
    }
}

pub fn parse(input: &str) -> Value {
    let mut tokens = Tokenizer {
        chars: input.chars().collect(),
        position: 0,
    };
    let mut root = HashMap::new();
    while let Some(key) = tokens.next_token() {
        if key == "}" {
            continue;
        }
        let value = tokens.parse_value();
        root.insert(key, value);
    }
    Value::Object(root)
}

struct Tokenizer {
    chars: Vec<char>,
    position: usize,
}

impl Tokenizer {
    fn parse_value(&mut self) -> Value {
        match self.next_token() {
            Some(token) if token == "{" => {
                let mut map = HashMap::new();
                loop {
                    match self.next_token() {
                        Some(key) if key == "}" => break,
                        Some(key) => {
                            let value = self.parse_value();
                            map.insert(key, value);
                        }
                        None => break,
                    }
                }
                Value::Object(map)
            }
            Some(token) => Value::String(token),
            None => Value::String(String::new()),
        }
    }

    fn next_token(&mut self) -> Option<String> {
        loop {
            while self
                .chars
                .get(self.position)
                .is_some_and(|c| c.is_whitespace())
            {
                self.position += 1;
            }
            // Skip `//` comments.
            if self.chars.get(self.position) == Some(&'/')
                && self.chars.get(self.position + 1) == Some(&'/')
            {
                while self.chars.get(self.position).is_some_and(|c| *c != '\n') {
                    self.position += 1;
                }
                continue;
            }
            break;
        }

        let current = *self.chars.get(self.position)?;
        if current == '"' {
            self.position += 1;
            let mut token = String::new();
            while let Some(&c) = self.chars.get(self.position) {
                self.position += 1;
                match c {
                    '"' => return Some(token),
                    '\\' => {
                        if let Some(&escaped) = self.chars.get(self.position) {
                            self.position += 1;
                            token.push(match escaped {
                                'n' => '\n',
                                't' => '\t',
                                other => other,
                            });
                        }
                    }
                    other => token.push(other),
                }
            }
            return Some(token);
        }

        if current == '{' || current == '}' {
            self.position += 1;
            return Some(current.to_string());
        }

        let mut token = String::new();
        while let Some(&c) = self.chars.get(self.position) {
            if c.is_whitespace() || c == '{' || c == '}' {
                break;
            }
            token.push(c);
            self.position += 1;
        }
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_app_manifest() {
        let input = r#"
        "AppState"
        {
            "appid"     "1174180"
            "name"      "Red Dead Redemption 2"
            "installdir"    "Red Dead Redemption 2"
        }
        "#;
        let parsed = parse(input);
        let state = parsed.get("AppState").expect("AppState");
        assert_eq!(state.get("appid").and_then(Value::as_str), Some("1174180"));
        assert_eq!(
            state.get("name").and_then(Value::as_str),
            Some("Red Dead Redemption 2")
        );
    }
}
