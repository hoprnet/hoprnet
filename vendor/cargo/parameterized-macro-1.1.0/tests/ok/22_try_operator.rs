use parameterized_macro::parameterized;

#[parameterized(v = { 1, 2 })]
fn my_test(v: i32) -> Result<(), ()> {
    let unit = Err(())?;
    Ok(unit)
}

fn main() {}
