use parameterized_macro::parameterized;

#[parameterized(v = {"a", "b"}, w={1,2})]
fn my_test(v: &str, w: i32) {}

fn main() {}
