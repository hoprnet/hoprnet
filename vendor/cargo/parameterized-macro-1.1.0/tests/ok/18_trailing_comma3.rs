use parameterized_macro::parameterized;

// a trailing comma after w's arguments (multiple inputs)
#[parameterized(
    v = { 1, 2 },
    w = { 1, 2 },
)]
fn my_test(v: u32, w: u32) {}

fn main() {}
