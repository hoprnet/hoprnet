#[cfg(feature = "sqlite")]
pub mod sqlite {
    // Include the generated SQLite entities from OUT_DIR
    include!(concat!(env!("OUT_DIR"), "/codegen/sqlite/mod.rs"));
}
