use parameterized_macro::parameterized;

#[parameterized(number = { 1, 2, 3 })]
#[should_panic]
fn my_test(number: i32) {}

fn main() {}
