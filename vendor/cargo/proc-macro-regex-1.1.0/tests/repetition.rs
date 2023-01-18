use proc_macro_regex::regex;

#[test]
fn zero_or_more_1() {
    regex!(regex "a*");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("b"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn zero_or_more_2() {
    regex!(regex "^a*");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn zero_or_more_3() {
    regex!(regex "a*$");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn zero_or_more_4() {
    regex!(regex "^a*$");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("ba"));
}

#[test]
fn zero_or_one_1() {
    regex!(regex "a?");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("b"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn zero_or_one_2() {
    regex!(regex "^a?");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("b"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn zero_or_one_3() {
    regex!(regex "a?$");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("b"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn zero_or_one_4() {
    regex!(regex "^a?$");
    assert!(regex(""));
    assert!(regex("a"));
    assert!(!regex("aa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("ba"));
}

#[test]
fn one_or_more_1() {
    regex!(regex "a+");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(!regex("b"));
    assert!(regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn one_or_more_2() {
    regex!(regex "^a+");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(!regex("b"));
    assert!(regex("ab"));
    assert!(!regex("ba"));
}

#[test]
fn one_or_more_3() {
    regex!(regex "a+$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(regex("ba"));
}

#[test]
fn one_or_more_4() {
    regex!(regex "^a+$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("ba"));
}

#[test]
fn range_exactly_1() {
    regex!(regex "a{2}");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(regex("aab"));
    assert!(regex("aaab"));
    assert!(!regex("ba"));
    assert!(regex("baa"));
    assert!(regex("baaa"));
}

#[test]
fn range_exactly_2() {
    regex!(regex "^a{2}");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(regex("aab"));
    assert!(regex("aaab"));
    assert!(!regex("ba"));
    assert!(!regex("baa"));
    assert!(!regex("baaa"));
}

#[test]
fn range_exactly_3() {
    regex!(regex "a{2}$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("aab"));
    assert!(!regex("aaab"));
    assert!(!regex("ba"));
    assert!(regex("baa"));
    assert!(regex("baaa"));
}

#[test]
fn range_exactly_4() {
    regex!(regex "^a{2}$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(!regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("aab"));
    assert!(!regex("aaab"));
    assert!(!regex("ba"));
    assert!(!regex("baa"));
    assert!(!regex("baaa"));
}

#[test]
fn range_at_least_1() {
    regex!(regex "a{2,}");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(regex("aab"));
    assert!(regex("aaab"));
    assert!(!regex("ba"));
    assert!(regex("baa"));
    assert!(regex("baaa"));
}

#[test]
fn range_at_least_2() {
    regex!(regex "^a{2,}");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(regex("aab"));
    assert!(regex("aaab"));
    assert!(!regex("ba"));
    assert!(!regex("baa"));
    assert!(!regex("baaa"));
}

#[test]
fn range_at_least_3() {
    regex!(regex "a{2,}$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("aab"));
    assert!(!regex("aaab"));
    assert!(!regex("ba"));
    assert!(regex("baa"));
    assert!(regex("baaa"));
}

#[test]
fn range_at_least_4() {
    regex!(regex "^a{2,}$");
    assert!(!regex(""));
    assert!(!regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("aab"));
    assert!(!regex("aaab"));
    assert!(!regex("ba"));
    assert!(!regex("baa"));
    assert!(!regex("baaa"));
}

#[test]
fn range_bounded_1() {
    regex!(regex "a{1,4}");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(regex("aaaa"));
    assert!(regex("aaaaa"));
    assert!(!regex("b"));
    assert!(regex("ab"));
    assert!(regex("aab"));
    assert!(regex("aaab"));
    assert!(regex("aaaab"));
    assert!(regex("aaaaab"));
    assert!(regex("ba"));
    assert!(regex("baa"));
    assert!(regex("baaa"));
    assert!(regex("baaaa"));
    assert!(regex("baaaaa"));
}

#[test]
fn range_bounded_2() {
    regex!(regex "^a{1,4}");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(regex("aaaa"));
    assert!(regex("aaaaa"));
    assert!(!regex("b"));
    assert!(regex("ab"));
    assert!(regex("aab"));
    assert!(regex("aaab"));
    assert!(regex("aaaab"));
    assert!(regex("aaaaab"));
    assert!(!regex("ba"));
    assert!(!regex("baa"));
    assert!(!regex("baaa"));
    assert!(!regex("baaaa"));
    assert!(!regex("baaaaa"));
}

#[test]
fn range_bounded_3() {
    regex!(regex "a{1,4}$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(regex("aaaa"));
    assert!(regex("aaaaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("aab"));
    assert!(!regex("aaab"));
    assert!(!regex("aaaab"));
    assert!(!regex("aaaaab"));
    assert!(regex("ba"));
    assert!(regex("baa"));
    assert!(regex("baaa"));
    assert!(regex("baaaaa"));
}

#[test]
fn range_bounded_4() {
    regex!(regex "^a{1,4}$");
    assert!(!regex(""));
    assert!(regex("a"));
    assert!(regex("aa"));
    assert!(regex("aaa"));
    assert!(regex("aaaa"));
    assert!(!regex("aaaaa"));
    assert!(!regex("b"));
    assert!(!regex("ab"));
    assert!(!regex("aab"));
    assert!(!regex("aaab"));
    assert!(!regex("aaaab"));
    assert!(!regex("aaaaab"));
    assert!(!regex("ba"));
    assert!(!regex("baa"));
    assert!(!regex("baaa"));
    assert!(!regex("baaaa"));
    assert!(!regex("baaaaa"));
}
