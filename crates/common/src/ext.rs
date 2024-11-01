pub trait RustNaming {
    fn to_rust_module(&self) -> String;
    fn to_rust_const(&self) -> String;
    fn to_camel_case(&self) -> String;
}

impl RustNaming for str {
    fn to_rust_module(&self) -> String {
        self.replace('-', "_")
    }

    fn to_rust_const(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for (i, char) in self.chars().enumerate() {
            if char == '.' || char == '-' {
                s.push('_');
                continue;
            } else if char == '_' {
                // allowed
            } else if !char.is_ascii_alphanumeric() {
                panic!("Only ascii chars and '.' allowed in rust constant names, not {char}")
            }
            if char.is_ascii_uppercase() && i != 0 {
                s.push('_');
                s.push(char);
            } else {
                s.push(char.to_ascii_uppercase());
            }
        }
        s
    }

    fn to_camel_case(&self) -> String {
        let mut s = String::with_capacity(self.len());
        let mut uppercase = true;
        for char in self.chars() {
            if s.is_empty() && (char.is_ascii_digit() || char == '-' || char == '.' || char == '_')
            {
                continue;
            } else if char == '.' || char == '_' || char == '-' {
                uppercase = true;
                continue;
            } else if char.is_ascii_alphanumeric() {
                if uppercase {
                    s.push(char.to_ascii_uppercase());
                    uppercase = false;
                } else {
                    s.push(char);
                }
            }
        }
        s
    }
}

pub trait StringExt {
    fn prefixed(&self, ch: char) -> String;
    fn postfixed(&self, ch: char) -> String;
}

impl StringExt for str {
    fn prefixed(&self, ch: char) -> String {
        format!("{ch}{self}")
    }
    fn postfixed(&self, ch: char) -> String {
        format!("{self}{ch}")
    }
}

pub trait OptStringExt {
    fn prefixed_or_default(&self, ch: char) -> String;
    fn postfixed_or_default(&self, ch: char) -> String;
}

impl OptStringExt for Option<String> {
    fn prefixed_or_default(&self, ch: char) -> String {
        self.as_ref().map(|s| s.prefixed(ch)).unwrap_or_default()
    }
    fn postfixed_or_default(&self, ch: char) -> String {
        self.as_ref().map(|s| s.postfixed(ch)).unwrap_or_default()
    }
}

impl OptStringExt for Option<&str> {
    fn prefixed_or_default(&self, ch: char) -> String {
        self.map(|s| s.prefixed(ch)).unwrap_or_default()
    }
    fn postfixed_or_default(&self, ch: char) -> String {
        self.map(|s| s.postfixed(ch)).unwrap_or_default()
    }
}
