use parameterized_macro::parameterized;

#[parameterized(x = { 0, 1 })]
fn my_test(x: i32, y: i32) {}

fn main() {}
