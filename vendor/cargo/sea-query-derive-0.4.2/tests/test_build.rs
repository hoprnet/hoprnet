#[test]
fn build_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("./tests/compile-fail/*.rs");
    t.compile_fail("./tests/compile-fail/enum_def/*.rs");

    // all of these are exactly the same as the examples in `examples/derive.rs`
    t.pass("./tests/pass/*.rs");
    t.pass("./tests/pass/enum_def/*.rs");
    t.pass("./tests/pass-static/*.rs");
}
