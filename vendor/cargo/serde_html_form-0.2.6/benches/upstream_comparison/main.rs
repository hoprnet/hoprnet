use serde::Deserialize;

fn main() {
    divan::main();
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct StructForm {
    foo: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct StructForm2 {
    optional_field: Option<u64>,
    float_needs_more_complex_parsing_and_has_very_long_field_name: f64,
}

#[derive(Deserialize)]
enum SimpleEnum {
    VariantA,
    VariantB,
    VariantC,
}

// For rust-analyzer, treat `benches.rs` as a regular module.
// The module itself also contains some `cfg(rust_analyzer)` code to include
// the symbols otherwise introduced through the inline modules below.
#[cfg(rust_analyzer)]
mod benches;

// For actual execution, treat `benches.rs` as two modules, one where `from_str`
// from `serde_html_form` is benchmarked, one where `from_str` from
// `serde_urlencoded` is benchmarked.
#[cfg(not(rust_analyzer))]
mod serde_html_form {
    use serde_html_form::from_str;
    include!("benches.rs");
}

#[cfg(not(rust_analyzer))]
mod serde_urlencoded {
    use serde_html_form::from_str;
    include!("benches.rs");
}
