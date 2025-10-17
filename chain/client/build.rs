fn main() {
    cynic_codegen::register_schema("blokli")
        .from_sdl_file("schemas/blokli-schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}