use parameterized_macro::parameterized;

enum Color {
    Red(Pigment),
    Yellow,
    Blue(Pigment),
}

struct Pigment {
    material_id: u32,
}

impl Pigment {
    fn new(id: u32) -> Self {
        Self { material_id: id }
    }
}

impl Default for Pigment {
    fn default() -> Self {
        Self { material_id: 0 }
    }
}

#[parameterized(v = { Color::Red(Pigment::new(5)), Color::Yellow,  Color::Blue::default(), Color::Red(Pigment {
    material_id: 8
}) })]
fn my_test(v: Color) {}

fn main() {}
