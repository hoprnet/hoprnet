/// Check if input is a valid domain label
pub fn is_label(input: &str) -> bool {
    let mut chars = input.chars();

    // we need at least one char
    let first = match chars.next() {
        None => {
            return false;
        }
        Some(c) => c,
    };

    // it can start with an alphanumeric character
    if !first.is_ascii_alphanumeric() {
        return false;
    }

    // then optionally be followed by any combination of
    // alphanumeric characters and dashes
    let last_index = input.len() - 2.min(input.len());
    for (index, c) in chars.enumerate() {
        // before finally ending with an alphanumeric character
        if !c.is_ascii_alphanumeric() && (index == last_index || c != '-') {
            return false;
        }
    }

    true
}

/// Check the local part of an email address (before @)
pub fn is_email_local(input: &str) -> bool {
    let mut chars = input.chars();

    // we need at least one char
    let first = match chars.next() {
        None => {
            return false;
        }
        Some(c) => c,
    };

    let last_index = input.len() - 2.min(input.len());
    if first == ' ' {
        return false;
    } else if first == '"' {
        // quoted
        if input.len() == 1 {
            return false;
        }
        for (index, c) in chars.enumerate() {
            if index == last_index {
                if c != '"' {
                    return false;
                }
            } else if !is_combined(c) && !is_quoted(c) {
                return false;
            }
        }
    } else {
        // not quoted
        if first == '.' {
            return false;
        }
        for (index, c) in chars.enumerate() {
            if !is_combined(c) && (index == last_index || c != '.') {
                return false;
            }
        }
    }

    true
}

// these characters can be anywhere in the expresion
// [[:alnum:]!#$%&'*+/=?^_`{|}~-]
fn is_global(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '-'
        || c == '!'
        || c == '#'
        || c == '$'
        || c == '%'
        || c == '&'
        || c == '\''
        || c == '*'
        || c == '+'
        || c == '/'
        || c == '='
        || c == '?'
        || c == '^'
        || c == '_'
        || c == '`'
        || c == '{'
        || c == '|'
        || c == '}'
        || c == '~'
}
fn is_non_ascii(c: char) -> bool {
    c as u32 > 0x7f // non-ascii characters (can also be unquoted)
}
fn is_quoted(c: char) -> bool {
    // ["(),\\:;<>@\[\]. ]
    c == '"'
        || c == '.'
        || c == ' '
        || c == '('
        || c == ')'
        || c == ','
        || c == '\\'
        || c == ':'
        || c == ';'
        || c == '<'
        || c == '>'
        || c == '@'
        || c == '['
        || c == ']'
}
fn is_combined(c: char) -> bool {
    is_global(c) || is_non_ascii(c)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_label_correct() {
        for l in &["a", "ab", "a-b", "a--b", "0Z"] {
            assert!(is_label(l));
        }
    }

    #[test]
    fn is_label_incorrect() {
        for l in &["", "-", "a-", "-b", "$"] {
            assert!(!is_label(l));
        }
    }

    #[test]
    fn is_email_local_correct() {
        for l in &[
            "a",
            "ab",
            "a.b",
            "a\u{0080}",
            "$",
            "\"\"\"",
            "\"a b\"",
            "\" \"",
            "\"a<>@\"",
        ] {
            assert!(is_email_local(l));
        }
    }

    #[test]
    fn is_email_local_incorrect() {
        for l in &["", " a", "a ", "a.", ".b", "a\x7f", "\"", "\"a", "a\""] {
            assert!(!is_email_local(l));
        }
    }
}
