use parameterized_macro::parameterized;

#[parameterized(zzz = { "a", "b" }, aaa = { 1, 2, 3 })]
pub(crate) fn my_test(v: &str, w: i32) {}

fn main() {}
