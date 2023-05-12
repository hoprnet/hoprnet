use proc_macro_regex::regex;

#[test]
fn character_class_regex() {
    regex!(character_class "[xyz]");
    assert!(character_class("x"));
    assert!(!character_class("a"));
}

#[test]
fn character_class_except_regex() {
    regex!(character_class_except b"[^x]");
    assert!(character_class_except(b"a"));
    assert!(!character_class_except(b"x"));
}

#[test]
fn character_class_range_regex() {
    regex!(character_class_range "[a-c]");
    assert!(character_class_range("a"));
    assert!(character_class_range("c"));
    assert!(!character_class_range("x"));
}

#[test]
fn character_class_alpha_regex() {
    regex!(character_class_alpha "[[:alpha:]]");
    assert!(character_class_alpha("a"));
    assert!(character_class_alpha("Z"));
    assert!(!character_class_alpha("1"));
}

#[test]
fn character_class_nested_regex() {
    regex!(character_class_nested b"[x[^xyz]]");
    assert!(character_class_nested(b"x"));
    assert!(!character_class_nested(b"y"));
}
