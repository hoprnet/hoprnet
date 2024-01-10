use parameterized_macro::parameterized as pm;

#[pm(v = { (), () })]
fn my_test(v: ()) {}

fn main() {}
