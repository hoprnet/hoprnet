use std::collections::BTreeMap;

// For rust-analyzer, this file is included as a regular module. Make IDE
// functionality work by pretending that the symbols being tested always come
// from `serde_html_form`.
#[cfg(rust_analyzer)]
use serde_html_form::from_str;

use crate::{SimpleEnum, StructForm, StructForm2};

#[divan::bench]
fn deserialize_list_enum() {
    from_str::<Vec<(u64, SimpleEnum)>>("10=VariantB&5=VariantC&2=VariantA&1=VariantA").unwrap();
}

#[divan::bench]
fn deserialize_list_duplicate_keys() {
    from_str::<Vec<(&str, &str)>>("a=test&b=test&a=test&b=test").unwrap();
}

#[divan::bench]
fn deserialize_map() {
    from_str::<BTreeMap<&str, i32>>("a=0&bb=1&ccc=123").unwrap();
}

#[divan::bench]
fn deserialize_map_many_entries() {
    from_str::<BTreeMap<i32, i32>>(
        "0=0&1=1&2=2&3=3&4=4&5=5&6=6&7=7&8=8&\
         200=0&201=1&202=2&203=3&204=4&205=5&206=6&207=7&208=8&\
         50=0&51=1&52=2&53=3&54=4&55=5&56=6&57=7&58=8&\
         1230=0&1231=1&1232=2&1233=3&1234=4&1235=5&1236=6&1237=7&1238=8&\
         80=0&81=1&82=2&83=3&84=4&85=5&86=6&87=7&88=8",
    )
    .unwrap();
}

#[divan::bench]
fn deserialize_struct_simple() {
    from_str::<StructForm>("foo=value").unwrap();
}

#[divan::bench]
fn deserialize_struct_long_parameters() {
    from_str::<StructForm2>("float_needs_more_complex_parsing_and_has_very_long_field_name=10")
        .unwrap();
}

#[divan::bench]
fn deserialize_struct_long_parameters_2() {
    from_str::<StructForm2>(
        "float_needs_more_complex_parsing_and_has_very_long_field_name=1.0000000000123&\
         optional_field=12300000000000000",
    )
    .unwrap();
}
