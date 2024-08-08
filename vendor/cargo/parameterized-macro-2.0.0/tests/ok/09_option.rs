use parameterized_macro::parameterized;

#[parameterized(v = { Some(-1), None })]
fn my_test(v: Option<i32>) {}

fn main() {}
