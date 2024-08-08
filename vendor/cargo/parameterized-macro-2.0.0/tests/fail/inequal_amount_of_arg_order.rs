use parameterized_macro::parameterized;

#[parameterized(b = { "a", "b", "c", "d" }, a = { 1, 2, 3 })]
pub(crate) fn my_test(b: &str, a: i32) {}

fn main() {}
