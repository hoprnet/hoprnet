use parameterized_macro::parameterized;

#[parameterized(
    v = {"a", "b"},     // test with &str inputs
    w = {1, 2}          // test with i32 inputs
)]
fn my_test(v: &str, w: i32) {}

fn main() {}
