use parameterized_macro::parameterized;

#[parameterized(x = { 1, 2, 3 }, y = { 1, 2, 3 })]
fn my_test(x: i32, x2: i32) {}

fn main() {}
