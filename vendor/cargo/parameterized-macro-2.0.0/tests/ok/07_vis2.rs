// see also 'tests/fail/on_visibility.rs'

#[macro_use]
extern crate parameterized_macro;

pub mod a {
    #[parameterized(v = { Some(- 1), None })]
    pub(in crate::b) fn my_test(v: Option<i32>) {}
}

mod b {
    #[cfg(test)]
    fn call() {
        a::my_test::case_0(); // this is ok
    }
}

fn main() {}
