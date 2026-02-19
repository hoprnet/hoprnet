fn main() {
    println!("cargo:rerun-if-env-changed=FIXED_RNG");
    if let Ok(val) = std::env::var("FIXED_RNG")
        && val.to_lowercase() == "true"
    {
        println!("cargo:rustc-cfg=feature=\"fixed_rng\"");
    }
}
