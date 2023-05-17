use more_asserts as ma;
use std::panic::catch_unwind;

#[derive(PartialOrd, PartialEq, Debug)]
enum DummyType {
    Foo,
    Bar,
    Baz,
}

#[test]
fn test_assert_lt() {
    ma::assert_lt!(3, 4);
    ma::assert_lt!(4.0, 4.5);
    ma::assert_lt!("a string", "b string");
    ma::assert_lt!(
        DummyType::Foo,
        DummyType::Bar,
        "Message with {}",
        "cool formatting"
    );

    let a = &DummyType::Foo;
    let b = &DummyType::Baz;
    ma::assert_lt!(a, b);

    assert!(catch_unwind(|| ma::assert_lt!(5, 3)).is_err());
    assert!(catch_unwind(|| ma::assert_lt!(5, 5)).is_err());
    assert!(catch_unwind(|| ma::assert_lt!(DummyType::Bar, DummyType::Foo)).is_err());
}

#[test]
fn test_assert_gt() {
    ma::assert_gt!(4, 3);
    ma::assert_gt!(4.5, 4.0);
    ma::assert_gt!("b string", "a string");
    ma::assert_gt!(
        DummyType::Bar,
        DummyType::Foo,
        "Message with {}",
        "cool formatting"
    );

    let a = &DummyType::Foo;
    let b = &DummyType::Baz;
    ma::assert_gt!(b, a);

    assert!(catch_unwind(|| ma::assert_gt!(3, 5)).is_err());
    assert!(catch_unwind(|| ma::assert_gt!(5, 5)).is_err());
    assert!(catch_unwind(|| ma::assert_gt!(DummyType::Foo, DummyType::Bar)).is_err());
}

#[test]
fn test_assert_le() {
    ma::assert_le!(3, 4);
    ma::assert_le!(4, 4);
    ma::assert_le!(4.0, 4.5);
    ma::assert_le!("a string", "a string");
    ma::assert_le!("a string", "b string");
    ma::assert_le!(DummyType::Foo, DummyType::Bar, "Message");
    ma::assert_le!(
        DummyType::Foo,
        DummyType::Foo,
        "Message with {}",
        "cool formatting"
    );

    let a = &DummyType::Foo;
    let b = &DummyType::Baz;
    ma::assert_le!(a, a);
    ma::assert_le!(a, b);

    assert!(catch_unwind(|| ma::assert_le!(5, 3)).is_err());
    assert!(catch_unwind(|| ma::assert_le!(DummyType::Bar, DummyType::Foo)).is_err());
}

#[test]
fn test_assert_ge() {
    ma::assert_ge!(4, 3);
    ma::assert_ge!(4, 4);
    ma::assert_ge!(4.5, 4.0);
    ma::assert_ge!(5.0, 5.0);
    ma::assert_ge!("a string", "a string");
    ma::assert_ge!("b string", "a string");
    ma::assert_ge!(DummyType::Bar, DummyType::Bar, "Example");
    ma::assert_ge!(
        DummyType::Bar,
        DummyType::Foo,
        "Message with {}",
        "cool formatting",
    );

    let a = &DummyType::Foo;
    let b = &DummyType::Baz;
    ma::assert_ge!(a, a);
    ma::assert_ge!(b, a);

    assert!(catch_unwind(|| ma::assert_ge!(3, 5)).is_err());
    assert!(catch_unwind(|| ma::assert_ge!(DummyType::Foo, DummyType::Bar)).is_err());
}
