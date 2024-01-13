// Only needed to find the README to use `include_str!` on it.

fn main() {
	// Go upwards until we find a README.md
	let mut path = std::env::current_dir().unwrap();
	while !path.join("README.md").exists() {
		path = path.parent().unwrap().to_path_buf();
	}
	path = path.join("README.md");
	// Sanity check that it contains the string "Proc Macro Warning".
	let contents = std::fs::read_to_string(&path).unwrap();
	assert!(contents.contains("Proc Macro Warning"));

	println!("cargo:rustc-env=README_PATH={}", path.display());
}
