// fixme: we don't support this feature yet, so anything below is speculation about what it might
//        look like
//use parameterized_macro::parameterized;
//
//// Vec or something else that implements IntoIter<Item=T>
//fn f() -> Vec<Option<i32>> {
//    vec![Some(1), Some(2)]
//}
//
//fn g() -> Vec<Result<i32, ()>> {
//    vec![Err(())]
//}
//
///// Say we define a function with identifier `five`:
///// ```
///// fn five() -> Vec<i8> { vec![5, 5, 5, 5] }
///// ```
///// To write a parameterized test which uses this function to generate inputs,
///// we take the id of the function: `five` and use that in our test case:
/////
///// ```
///// #[parameterized(fn = five)]
///// fn my_test_case(five: i8) {
/////   assert!(five == 5)
///// }
///// ```
///// The parameter of your defined test case function should be the Item value (T) of the
///// IntoIterator<Item=T>.
/////
//#[parameterized(fn = {f, g})]
//fn my_test(f: Option<i32>, g: Result<i32, ()>) {
//    assert!(f.is_ok() && g.is_err());
//}
//
fn main() {}
