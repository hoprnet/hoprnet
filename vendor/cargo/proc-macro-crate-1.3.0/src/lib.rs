/*!

[![](https://docs.rs/proc-macro-crate/badge.svg)](https://docs.rs/proc-macro-crate/) [![](https://img.shields.io/crates/v/proc-macro-crate.svg)](https://crates.io/crates/proc-macro-crate) [![](https://img.shields.io/crates/d/proc-macro-crate.png)](https://crates.io/crates/proc-macro-crate) [![Build Status](https://travis-ci.org/bkchr/proc-macro-crate.png?branch=master)](https://travis-ci.org/bkchr/proc-macro-crate)

Providing support for `$crate` in procedural macros.

* [Introduction](#introduction)
* [Example](#example)
* [License](#license)

## Introduction

In `macro_rules!` `$crate` is used to get the path of the crate where a macro is declared in. In
procedural macros there is currently no easy way to get this path. A common hack is to import the
desired crate with a know name and use this. However, with rust edition 2018 and dropping
`extern crate` declarations from `lib.rs`, people start to rename crates in `Cargo.toml` directly.
However, this breaks importing the crate, as the proc-macro developer does not know the renamed
name of the crate that should be imported.

This crate provides a way to get the name of a crate, even if it renamed in `Cargo.toml`. For this
purpose a single function `crate_name` is provided. This function needs to be called in the context
of a proc-macro with the name of the desired crate. `CARGO_MANIFEST_DIR` will be used to find the
current active `Cargo.toml` and this `Cargo.toml` is searched for the desired crate.

## Example

```
use quote::quote;
use syn::Ident;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};

fn import_my_crate() {
    let found_crate = crate_name("my-crate").expect("my-crate is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => quote!( crate::Something ),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident::Something )
        }
    };
}

# fn main() {}
```

## Edge cases

There are multiple edge cases when it comes to determining the correct crate. If you for example
import a crate as its own dependency, like this:

```toml
[package]
name = "my_crate"

[dev-dependencies]
my_crate = { version = "0.1", features = [ "test-feature" ] }
```

The crate will return `FoundCrate::Itself` and you will not be able to find the other instance
of your crate in `dev-dependencies`. Other similar cases are when one crate is imported multiple
times:

```toml
[package]
name = "my_crate"

[dependencies]
some-crate = { version = "0.5" }
some-crate-old = { package = "some-crate", version = "0.1" }
```

When searching for `some-crate` in this `Cargo.toml` it will return `FoundCrate::Name("some_old_crate")`,
aka the last definition of the crate in the `Cargo.toml`.

## License

Licensed under either of

 * [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)

 * [MIT license](https://opensource.org/licenses/MIT)

at your option.
*/

use std::{
<<<<<<<< HEAD:vendor/cargo/proc-macro-crate-1.1.3/src/lib.rs
    collections::HashMap,
    env,
    fs::File,
    io::{self, Read},
========
    collections::btree_map::{self, BTreeMap},
    env, fmt, fs, io,
>>>>>>>> master:vendor/cargo/proc-macro-crate-1.3.0/src/lib.rs
    path::{Path, PathBuf},
};

<<<<<<<< HEAD:vendor/cargo/proc-macro-crate-1.1.3/src/lib.rs
use toml::{self, value::Table};
========
use once_cell::sync::Lazy;
use toml_edit::{Document, Item, Table, TomlError};
>>>>>>>> master:vendor/cargo/proc-macro-crate-1.3.0/src/lib.rs

type CargoToml = HashMap<String, toml::Value>;

/// Error type used by this crate.
#[derive(Debug)]
pub enum Error {
    NotFound(PathBuf),
    CargoManifestDirNotSet,
    CouldNotRead { path: PathBuf, source: io::Error },
    InvalidToml { source: TomlError },
    CrateNotFound { crate_name: String, path: PathBuf },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::CouldNotRead { source, .. } => Some(source),
            Error::InvalidToml { source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotFound(path) => write!(
                f,
                "Could not find `Cargo.toml` in manifest dir: `{}`.",
                path.display()
            ),
            Error::CargoManifestDirNotSet => {
                f.write_str("`CARGO_MANIFEST_DIR` env variable not set.")
            }
            Error::CouldNotRead { path, .. } => write!(f, "Could not read `{}`.", path.display()),
            Error::InvalidToml { .. } => f.write_str("Invalid toml file."),
            Error::CrateNotFound { crate_name, path } => write!(
                f,
                "Could not find `{}` in `dependencies` or `dev-dependencies` in `{}`!",
                crate_name,
                path.display(),
            ),
        }
    }
}

/// The crate as found by [`crate_name`].
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum FoundCrate {
    /// The searched crate is this crate itself.
    Itself,
    /// The searched crate was found with this name.
    Name(String),
}

/// Find the crate name for the given `orig_name` in the current `Cargo.toml`.
///
/// `orig_name` should be the original name of the searched crate.
///
/// The current `Cargo.toml` is determined by taking `CARGO_MANIFEST_DIR/Cargo.toml`.
///
/// # Returns
///
/// - `Ok(orig_name)` if the crate was found, but not renamed in the `Cargo.toml`.
/// - `Ok(RENAMED)` if the crate was found, but is renamed in the `Cargo.toml`. `RENAMED` will be
/// the renamed name.
/// - `Err` if an error occurred.
///
/// The returned crate name is sanitized in such a way that it is a valid rust identifier. Thus,
/// it is ready to be used in `extern crate` as identifier.
pub fn crate_name(orig_name: &str) -> Result<FoundCrate, Error> {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|_| Error::CargoManifestDirNotSet)?);

    let cargo_toml_path = manifest_dir.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Err(Error::NotFound(manifest_dir.into()));
    }

    let cargo_toml = open_cargo_toml(&cargo_toml_path)?;

    extract_crate_name(orig_name, cargo_toml, &cargo_toml_path)
}

/// Make sure that the given crate name is a valid rust identifier.
fn sanitize_crate_name<S: AsRef<str>>(name: S) -> String {
    name.as_ref().replace("-", "_")
}

/// Open the given `Cargo.toml` and parse it into a hashmap.
<<<<<<<< HEAD:vendor/cargo/proc-macro-crate-1.1.3/src/lib.rs
fn open_cargo_toml(path: &Path) -> Result<CargoToml, Error> {
    let mut content = String::new();
    File::open(path)
        .map_err(|e| Error::CouldNotRead {
            source: e,
            path: path.into(),
        })?
        .read_to_string(&mut content)
        .map_err(|e| Error::CouldNotRead {
            source: e,
            path: path.into(),
        })?;
    toml::from_str(&content).map_err(|e| Error::InvalidToml { source: e })
}

/// Extract the crate name for the given `orig_name` from the given `Cargo.toml` by checking the
/// `dependencies` and `dev-dependencies`.
///
/// Returns `Ok(orig_name)` if the crate is not renamed in the `Cargo.toml` or otherwise
/// the renamed identifier.
fn extract_crate_name(
    orig_name: &str,
    mut cargo_toml: CargoToml,
    cargo_toml_path: &Path,
) -> Result<FoundCrate, Error> {
    if let Some(toml::Value::Table(t)) = cargo_toml.get("package") {
        if let Some(toml::Value::String(s)) = t.get("name") {
            if s == orig_name {
                if std::env::var_os("CARGO_TARGET_TMPDIR").is_none() {
                    // We're running for a library/binary crate
                    return Ok(FoundCrate::Itself);
                } else {
                    // We're running for an integration test
                    return Ok(FoundCrate::Name(sanitize_crate_name(orig_name)));
                }
            }
        }
    }

    if let Some(name) = ["dependencies", "dev-dependencies"]
        .iter()
        .find_map(|k| search_crate_at_key(k, orig_name, &mut cargo_toml))
    {
        return Ok(FoundCrate::Name(sanitize_crate_name(name)));
    }

    // Start searching `target.xy.dependencies`
    if let Some(name) = cargo_toml
        .remove("target")
        .and_then(|t| t.try_into::<Table>().ok())
        .and_then(|t| {
            t.values()
                .filter_map(|v| v.as_table())
                .flat_map(|t| {
                    t.get("dependencies")
                        .into_iter()
                        .chain(t.get("dev-dependencies").into_iter())
                })
                .filter_map(|t| t.as_table())
                .find_map(|t| extract_crate_name_from_deps(orig_name, t.clone()))
========
fn open_cargo_toml(path: &Path) -> Result<Document, Error> {
    let content = fs::read_to_string(path).map_err(|e| Error::CouldNotRead {
        source: e,
        path: path.into(),
    })?;
    content
        .parse::<Document>()
        .map_err(|e| Error::InvalidToml { source: e })
}

/// Extract all crate names from the given `Cargo.toml` by checking the `dependencies` and
/// `dev-dependencies`.
fn extract_crate_names(cargo_toml: &Document) -> Result<CrateNames, Error> {
    let package_name = extract_package_name(cargo_toml);
    let root_pkg = package_name.as_ref().map(|name| {
        let cr = match env::var_os("CARGO_TARGET_TMPDIR") {
            // We're running for a library/binary crate
            None => FoundCrate::Itself,
            // We're running for an integration test
            Some(_) => FoundCrate::Name(sanitize_crate_name(name)),
        };

        (name.to_string(), cr)
    });

    let dep_tables = dep_tables(cargo_toml).chain(target_dep_tables(cargo_toml));
    let dep_pkgs = dep_tables.flatten().filter_map(move |(dep_name, dep_value)| {
        let pkg_name = dep_value
            .get("package")
            .and_then(|i| i.as_str())
            .unwrap_or(dep_name);

        if package_name.as_ref().map_or(false, |n| *n == pkg_name) {
            return None;
        }

        let cr = FoundCrate::Name(sanitize_crate_name(dep_name));

        Some((pkg_name.to_owned(), cr))
    });

    Ok(root_pkg.into_iter().chain(dep_pkgs).collect())
}

fn extract_package_name(cargo_toml: &Document) -> Option<&str> {
    cargo_toml.get("package")?.get("name")?.as_str()
}

fn target_dep_tables(cargo_toml: &Document) -> impl Iterator<Item = &Table> {
    cargo_toml
        .get("target")
        .into_iter()
        .filter_map(Item::as_table)
        .flat_map(|t| {
            t.iter()
                .map(|(_, value)| value)
                .filter_map(Item::as_table)
                .flat_map(dep_tables)
>>>>>>>> master:vendor/cargo/proc-macro-crate-1.3.0/src/lib.rs
        })
    {
        return Ok(FoundCrate::Name(sanitize_crate_name(name)));
    }

    Err(Error::CrateNotFound {
        crate_name: orig_name.into(),
        path: cargo_toml_path.into(),
    })
}

<<<<<<<< HEAD:vendor/cargo/proc-macro-crate-1.1.3/src/lib.rs
/// Search the `orig_name` crate at the given `key` in `cargo_toml`.
fn search_crate_at_key(key: &str, orig_name: &str, cargo_toml: &mut CargoToml) -> Option<String> {
    cargo_toml
        .remove(key)
        .and_then(|v| v.try_into::<Table>().ok())
        .and_then(|t| extract_crate_name_from_deps(orig_name, t))
}

/// Extract the crate name from the given dependencies.
///
/// Returns `Some(orig_name)` if the crate is not renamed in the `Cargo.toml` or otherwise
/// the renamed identifier.
fn extract_crate_name_from_deps(orig_name: &str, deps: Table) -> Option<String> {
    for (key, value) in deps.into_iter() {
        let renamed = value
            .try_into::<Table>()
            .ok()
            .and_then(|t| t.get("package").cloned())
            .map(|t| t.as_str() == Some(orig_name))
            .unwrap_or(false);

        if key == orig_name || renamed {
            return Some(key.clone());
        }
    }

    None
========
fn dep_tables(table: &Table) -> impl Iterator<Item = &Table> {
    table
        .get("dependencies")
        .into_iter()
        .chain(table.get("dev-dependencies"))
        .filter_map(Item::as_table)
>>>>>>>> master:vendor/cargo/proc-macro-crate-1.3.0/src/lib.rs
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! create_test {
        (
            $name:ident,
            $cargo_toml:expr,
            $( $result:tt )*
        ) => {
            #[test]
            fn $name() {
<<<<<<<< HEAD:vendor/cargo/proc-macro-crate-1.1.3/src/lib.rs
                let cargo_toml = toml::from_str($cargo_toml).expect("Parses `Cargo.toml`");
                let path = PathBuf::from("test-path");

                match extract_crate_name("my_crate", cargo_toml, &path) {
========
                let cargo_toml = $cargo_toml.parse::<Document>().expect("Parses `Cargo.toml`");

               match extract_crate_names(&cargo_toml).map(|mut map| map.remove("my_crate")) {
>>>>>>>> master:vendor/cargo/proc-macro-crate-1.3.0/src/lib.rs
                    $( $result )* => (),
                    o => panic!("Invalid result: {:?}", o),
                }
            }
        };
    }

    create_test! {
        deps_with_crate,
        r#"
            [dependencies]
            my_crate = "0.1"
        "#,
        Ok(FoundCrate::Name(name)) if name == "my_crate"
    }

    create_test! {
        dev_deps_with_crate,
        r#"
            [dev-dependencies]
            my_crate = "0.1"
        "#,
        Ok(FoundCrate::Name(name)) if name == "my_crate"
    }

    create_test! {
        deps_with_crate_renamed,
        r#"
            [dependencies]
            cool = { package = "my_crate", version = "0.1" }
        "#,
        Ok(FoundCrate::Name(name)) if name == "cool"
    }

    create_test! {
        deps_with_crate_renamed_second,
        r#"
            [dependencies.cool]
            package = "my_crate"
            version = "0.1"
        "#,
        Ok(FoundCrate::Name(name)) if name == "cool"
    }

    create_test! {
        deps_empty,
        r#"
            [dependencies]
        "#,
        Err(Error::CrateNotFound {
            crate_name,
            path,
        }) if crate_name == "my_crate" && path.display().to_string() == "test-path"
    }

    create_test! {
        crate_not_found,
        r#"
            [dependencies]
            serde = "1.0"
        "#,
        Err(Error::CrateNotFound {
            crate_name,
            path,
        }) if crate_name == "my_crate" && path.display().to_string() == "test-path"
    }

    create_test! {
        target_dependency,
        r#"
            [target.'cfg(target_os="android")'.dependencies]
            my_crate = "0.1"
        "#,
        Ok(FoundCrate::Name(name)) if name == "my_crate"
    }

    create_test! {
        target_dependency2,
        r#"
            [target.x86_64-pc-windows-gnu.dependencies]
            my_crate = "0.1"
        "#,
        Ok(FoundCrate::Name(name)) if name == "my_crate"
    }

    create_test! {
        own_crate,
        r#"
            [package]
            name = "my_crate"
        "#,
        Ok(FoundCrate::Itself)
    }

    create_test! {
        own_crate_and_in_deps,
        r#"
            [package]
            name = "my_crate"

            [dev-dependencies]
            my_crate = "0.1"
        "#,
        Ok(Some(FoundCrate::Itself))
    }

    create_test! {
        multiple_times,
        r#"
            [dependencies]
            my_crate = { version = "0.5" }
            my-crate-old = { package = "my_crate", version = "0.1" }
        "#,
        Ok(Some(FoundCrate::Name(name))) if name == "my_crate_old"
    }
}
