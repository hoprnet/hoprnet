use proc_macro_regex::regex;

#[test]
fn empty() {
    regex!(regex "");
    assert!(regex(""));
    assert!(regex("a"));
}

#[test]
fn literal_1() {
    regex!(regex "a");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn literal_2() {
    regex!(regex "^a");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("ab"));
    assert!(!regex("ba"));
}

#[test]
fn literal_3() {
    regex!(regex "a$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(!regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn literal_4() {
    regex!(regex "^a$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(!regex("ab"));
    assert!(!regex("ba"));
}

#[test]
fn class_1() {
    regex!(regex "[ab]");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("ab"));
    assert!(regex("abc"));
    assert!(regex("ba"));
    assert!(regex("cba"));
    assert!(regex("b"));
    assert!(!regex("c"));
}

#[test]
fn class_2() {
    regex!(regex "^[ab]");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("ab"));
    assert!(regex("abc"));
    assert!(regex("ba"));
    assert!(!regex("cba"));
    assert!(regex("b"));
    assert!(!regex("c"));
}

#[test]
fn class_3() {
    regex!(regex "[ab]$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("ab"));
    assert!(!regex("abc"));
    assert!(regex("ba"));
    assert!(regex("cba"));
    assert!(regex("b"));
    assert!(!regex("c"));
}

#[test]
fn class_4() {
    regex!(regex "^[ab]$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(!regex("ab"));
    assert!(!regex("abc"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(regex("b"));
    assert!(!regex("c"));
}

#[test]
fn alternation_1() {
    regex!(regex "ab|cb");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(regex("cb"));
    assert!(regex("abc"));
    assert!(regex("cab"));
    assert!(!regex("ba"));
    assert!(regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn alternation_2() {
    regex!(regex "^(ab|cb)");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(regex("cb"));
    assert!(regex("abc"));
    assert!(!regex("cab"));
    assert!(!regex("ba"));
    assert!(regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn alternation_3() {
    regex!(regex "(ab|cb)$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(regex("cb"));
    assert!(!regex("abc"));
    assert!(regex("cab"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn alternation_4() {
    regex!(regex "^(ab|cb)$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(regex("cb"));
    assert!(!regex("abc"));
    assert!(!regex("cab"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn concat_1() {
    regex!(regex "ab");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(regex("abc"));
    assert!(regex("cab"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn concat_2() {
    regex!(regex "^ab");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(regex("abc"));
    assert!(!regex("cab"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn concat_3() {
    regex!(regex "ab$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(!regex("abc"));
    assert!(regex("cab"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}

#[test]
fn concat_4() {
    regex!(regex "^ab$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("ab"));
    assert!(!regex("abc"));
    assert!(!regex("cab"));
    assert!(!regex("ba"));
    assert!(!regex("cba"));
    assert!(!regex("b"));
    assert!(!regex("c"));
}
