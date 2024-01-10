# parameterized

Procedural macro based parameterized testing library.
Useful, when you want to run a test case with many different input sets.

When defining a parameterized test case, the `#[parameterized(...)]` attribute should be used instead of `#[test]`.

This crate was inspired by JUnit `@ParameterizedTest`.

If you consider using parameterized, you can also check out [Yare](https://github.com/foresterre/yare) which is a variation on `parameterized`, which pivots the  parameters, so you can define your own identifier for cases. Alternatively, there is [Sif](https://github.com/foresterre/sif) where each case can be defined by a separate `#[case(...)`] attribute. 

### Examples:

Additional examples can be found at the <a href="https://github.com/foresterre/parameterized-examples">parameterized-examples repository</a>,
 and in the <a href="parameterized-macro/tests">tests</a> folder.

<br>

**Example: Fruits**

```rust
enum Fruit {
    Apple,
    Bramble(BrambleFruit),
    Pear,
}

trait NameOf {
    fn name_of(&self) -> &str;
}

impl NameOf for Fruit {
    fn name_of(&self) -> &str {
        match self {
            Fruit::Apple => "apple",
            Fruit::Bramble(fruit) => fruit.name_of(),
            Fruit::Pear => "pear",
        }
    }
}

enum BrambleFruit {
    Blackberry,
}

impl NameOf for BrambleFruit {
    fn name_of(&self) -> &str {
        match self {
            BrambleFruit::Blackberry => "blackberry",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;


    #[parameterized(fruit = {
        Fruit::Apple, Fruit::Pear, Fruit::Bramble(BrambleFruit::Blackberry)
    }, name = {
        "apple", "pear", "blackberry"
    })]
    fn a_fruity_test(fruit: Fruit, name: &str) {
        assert_eq!(fruit.name_of(), name)
    }
}
```

<br>

**Example: Add5**

```rust
fn add5<T: Into<u32>>(component: T) -> u32 {
    component.into() + 5
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;

    ide!();

    #[parameterized(input = {
        0, 1, 2
    }, expected = {
        5, 6, 7
    })]
    fn test_add5(input: u16, expected: u32) {
        assert_eq!(add5(input), expected);
    }
}
```

<br>

### Imports

If you prefer not to import this library (with `use parameterized::parameterized;`) in every test module, you can put
the following snippet at the top of your crate root:
```rust
#[cfg(test)]
#[macro_use]
extern crate parameterized;
```

<br>

### IDE 'run test' intent

IntelliJ IDEA recognizes test cases and provides context menus which allow you to run tests within a certain scope
(such as a module or a single test case). For example, in IntelliJ you can usually run individual test cases by clicking
the ▶ icon in the gutter. Unfortunately, attribute macros are currently not expanded by `intellij-rust`.
This means that the IDE will not recognize test cases generated as a result of attribute macros (such as the
`parameterized` macro published by this crate). 

A workaround can be found below (if you have a better solution, please feel free to open an issue; thank you in advance!)

```rust
fn squared(input: i8) -> i8 {
  input * input  
}

#[cfg(test)]
mod tests {
    use super::*;

    use parameterized::parameterized as pm;
    use parameterized::ide;
        
    mod squared_tests { // <--
        use super::*;

        ide!(); // <--
    
        #[pm(input = {
            -2, -1, 0, 1, 2
        }, expected = {
            4, 1, 0, 1, 4
        })]
        fn test_squared(input: i8, output: i8) {
            assert_eq(squared(input), output);
        }
    }
}
```

Here we created an empty test case (using the `ide!()` macro) which will mark the surrounding module as 'containing test cases'. In
the gutter you will find the ▶ icon next to the module. This allows you to run test cases per module.

Note: `intellij-rust` does expand declarative macro's (with the new macro engine which can be
selected in the 'settings' menu), such as this `ide!` macro.

<br>

### License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.</sub>
