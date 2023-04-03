use std::collections::HashMap;

pub fn compare_vec<T: std::fmt::Debug + Eq + std::hash::Hash>(a: &[T], b: &[T]) -> bool {
    let mut left_values: HashMap<&T, u32> = HashMap::new();
    for v in a.iter() {
        match left_values.get_mut(v) {
            None => {
                left_values.insert(v, 1);
            }
            Some(entry) => *entry += 1,
        }
    }

    let mut right_values: HashMap<&T, u32> = HashMap::new();
    for v in b.iter() {
        match right_values.get_mut(v) {
            None => {
                right_values.insert(v, 1);
            }
            Some(entry) => *entry += 1,
        }
    }
    left_values == right_values
}

/// Asserts that two expressions are equal to each other (using [`PartialEq`, `Hash`]).
///
/// On panic, this macro will print the values of the expressions with their
/// debug representations.
///
/// Like [`assert!`], this macro has a second form, where a custom
/// panic message can be provided.
///
/// [`PartialEq`]: cmp/trait.PartialEq.html
/// [`Hash`]: std/hash/trait.Hash.html
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate vector_assertions;
/// # fn main() {
///     let a = vec![1, 2];
///     let b = vec![2, 1];
///
///     assert_vec_eq!(a, b, "we are testing addition with {} and {}", "a", "b");
/// }
/// ```
#[macro_export]
macro_rules! assert_vec_eq {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_vec, right_vec) => {

                if !($crate::compare_vec(left_vec, right_vec)) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, &*left_vec, &*right_vec)
                }
            }
        }
    });
    ($left:expr, $right:expr,) => ({
        $crate::assert_eq!($left, $right)
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_vec, right_vec) => {
                if !($crate::compare_vec(left_vec, right_vec)) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, &*left_vec, &*right_vec,
                           std::format_args!($($arg)+))
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::compare_vec;

    #[test]
    fn compare_vec_simple() {
        let a = vec![1, 2];
        let b = vec![2, 1];
        let actual = compare_vec(&a, &b);
        assert!(actual);
    }

    #[test]
    fn it_works() {
        let a = vec![1, 2];
        let b = vec![2, 1];

        assert_vec_eq!(a, b);
        assert_vec_eq!(a, b, "we are testing addition with {} and {}", "a", "b");
    }
}
