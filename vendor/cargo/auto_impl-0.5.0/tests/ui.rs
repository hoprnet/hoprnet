use trybuild::TestCases;

#[test]
fn ui_compile_pass() {
    let t = TestCases::new();
    t.pass("tests/compile-pass/*.rs");
}

#[test]
fn ui_compile_fail() {
    let t = TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
