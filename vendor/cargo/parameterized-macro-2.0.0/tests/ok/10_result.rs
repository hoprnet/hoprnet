use parameterized_macro::parameterized;

#[parameterized(v = { OK(()), Err(()) })]
fn my_test(v: Result<(), ()>) {}

fn main() {}
