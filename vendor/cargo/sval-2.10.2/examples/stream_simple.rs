/*!
Streams have a simple core data-model that consists of:

- Nulls: the lack of any meaningful value.
- Booleans: `true` and `false`.
- Signed integers.
- Binary floating point numbers.
- Text strings.
- Sequences of values.

This example implements a simple stream that writes directly to stdout.
*/

pub mod stream;

fn main() -> sval::Result {
    stream(42)?;
    stream(true)?;

    stream(Some(42))?;
    stream(None::<i32>)?;

    #[cfg(feature = "alloc")]
    stream({
        use std::collections::BTreeMap;

        let mut map = BTreeMap::new();

        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        map
    })?;

    #[cfg(feature = "alloc")]
    stream(vec![vec!["Hello", "world"], vec!["Hello", "world"]])?;

    Ok(())
}

fn stream(v: impl sval::Value) -> sval::Result {
    v.stream(&mut stream::simple::MyStream)?;
    println!();

    Ok(())
}
