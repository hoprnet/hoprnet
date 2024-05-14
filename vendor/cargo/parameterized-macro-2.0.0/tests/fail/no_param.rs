use parameterized_macro::parameterized;

#[parameterized(y = { 1, 2, 3 })]
fn my_test(x: i32) {}

fn main() {}
