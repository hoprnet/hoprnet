use parameterized_macro::parameterized;

// a trailing comma after v's arguments
#[parameterized(
    v = { 1, 2 },
)]
fn my_test(v: u32) {}

fn main() {}
