use parameterized_macro::parameterized;

struct NotTry;

#[parameterized(_v = { 1, 2 })]
fn my_test(_v: i32) -> NotTry {
    NotTry
}

fn main() {}
