// fixme: presumably the following test won't succeed because the generated syntax is correct;
//  we need to run type checking before we know that this won't compile
//
//  If you move this test case to the 'expand' workspace crate, which is used to manually inspect
//  outcomes, you'll find that upon running `cargo test`, type checking does fail indeed.
//  If you inspect the token tree manually, you will also find that the 'expected cases' described below
//  are indeed generated correctly.

//#[cfg_attr(not(test), allow(unused))]
//use parameterized_macro::parameterized;
//
//#[cfg_attr(not(test), allow(unused))]
//enum Color {
//    Red,
//    Yellow,
//    Blue,
//}
//
//#[cfg_attr(not(test), allow(unused))]
//enum NotAColor {}
//
//#[parameterized(v = { Color::Red, Color::Yellow })]
//fn my_test(v: NotAColor) {}

// expected cases:
// ```
// fn #fn_ident () {
//   let v: NotAColor = Color::Red;
//   {} // body
// }
//
// fn #fn_ident () {
//   let v: NotAColor = Color::Yellow;
//   {} // body
// }
// ```

fn main() {}
