//! For docs see [here](https://docs.rs/crate/parameterized) :).
// FIXME: add rustdoc

pub use parameterized_macro::parameterized;

/// Attribute macro's such as 'parameterized' do not enable the run tests intent for a module
/// marked as cfg(test) (or a #[test] function for that matter) in Intellij.
///
/// To enable the intent within a module, we need at least a single test marked with `#[test]`.
/// The `ide!()` macro is a work around for this issue and creates this empty test. It can be called
/// within every module where we wish to run test cases using the run configuration / run test context
/// menu.
///
/// Using the intellij-rust new macro expansion engine, if this macro is called within a module,
/// the module will be marked as test, and the 'run as test' context menu will be provided in the
/// gutter.
#[macro_export]
macro_rules! ide {
    () => {
        #[test]
        fn __mark_with_test_intent() {}
    };
}

#[cfg(test)]
mod tests {
    use crate::ide;
    use crate::parameterized as pm;

    fn add5<T: Into<u32>>(component: T) -> u32 {
        component.into() + 5
    }

    mod readme_test {
        use super::*;

        ide!();

        #[pm(input = {
            0, 1, 2
        }, expected = {
            5, 6, 7
        })]
        fn test_add5(input: u16, expected: u32) {
            assert_eq!(add5(input), expected)
        }
    }

    mod marked_as_test_module {
        use super::*;

        ide!();

        #[pm(input = { 2, 3, 4 }, output = { 4, 6, 8 })]
        fn test_times2(input: i32, output: i32) {
            let times2 = |receiver: i32| receiver * 2;

            assert_eq!(times2(input), output);
        }
    }

    mod transitive_attrs {
        use super::*;

        ide!();

        #[pm(input = { None, None, None })]
        #[should_panic]
        fn numbers(input: Option<()>) {
            input.unwrap()
        }
    }

    mod result {
        use super::*;

        ide!();

        #[pm(input = { 2, 3, 4 }, output = { 2, 3, 4 })]
        fn ok(input: i32, output: i32) -> Result<(), ()> {
            let result = Ok(input)?;

            assert_eq!(result, output);

            Ok(())
        }

        #[pm(v = {
            Ok(1),
            // Err("Oh noes".to_string()), // Implements Termination, and reports the exit code: ExitCode::FAILURE, causing the test to fail! Uncomment line to see the result
        })]
        fn readme_example(v: Result<u32, String>) -> Result<(), String> {
            let value = v?; // Can use the question mark operator here, since return type is Result, which implements the Termination trait

            assert_eq!(value, 1);

            Ok(())
        }
    }
}
