use parameterized_macro::parameterized;

#[parameterized(v = { 1, 2 })]
fn my_test(v: i32) {}

fn main() {}
