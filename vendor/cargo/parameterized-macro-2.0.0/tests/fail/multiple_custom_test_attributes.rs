use parameterized_macro::parameterized;

#[parameterized(
    v = { 1, 2, 3, },
    w = { 1, 2, 3, },
)]
#[parameterized_macro(macro1)]
#[parameterized_macro(macro2)]
async fn my_test(v: u32, w: u32) {
    assert!(true);
}

fn main() {}
