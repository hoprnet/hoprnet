use parameterized_macro::parameterized;

enum Color {
    Red,
    Yellow,
    Blue,
}

#[parameterized(v = { Color::Red, Color::Yellow,  Color::Blue, Color::Red })]
fn my_test(v: Color) {}

fn main() {}
