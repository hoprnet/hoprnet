use proc_macro_regex::regex;

regex!(example_1 "abc");
regex!(example_2 "abc" 256);
regex!(pub example_3 "abc");
regex!(example_4 b"abc");

fn main() {
    println!("example_1 == {}", example_1("abc"));
    println!("example_2 == {}", example_2("abc"));
    println!("example_3 == {}", example_3("abc"));
    println!("example_4 == {}", example_4(b"abc"));
}
