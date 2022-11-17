extern crate cargo_metadata;
extern crate semver;
#[macro_use]
extern crate serde_json;

use cargo_metadata::{Metadata, MetadataCommand};
use std::path::PathBuf;

#[test]
fn old_minimal() {
    // Output from oldest supported version (1.24).
    // This intentionally has as many null fields as possible.
    // 1.8 is when metadata was introduced.
    // Older versions not supported because the following are required:
    // - `workspace_members` added in 1.13
    // - `target_directory` added in 1.19
    // - `workspace_root` added in 1.24
    let json = r#"
{
  "packages": [
    {
      "name": "foo",
      "version": "0.1.0",
      "id": "foo 0.1.0 (path+file:///foo)",
      "license": null,
      "license_file": null,
      "description": null,
      "source": null,
      "dependencies": [
        {
          "name": "somedep",
          "source": null,
          "req": "^1.0",
          "kind": null,
          "optional": false,
          "uses_default_features": true,
          "features": [],
          "target": null
        }
      ],
      "targets": [
        {
          "kind": [
            "bin"
          ],
          "crate_types": [
            "bin"
          ],
          "name": "foo",
          "src_path": "/foo/src/main.rs"
        }
      ],
      "features": {},
      "manifest_path": "/foo/Cargo.toml"
    }
  ],
  "workspace_members": [
    "foo 0.1.0 (path+file:///foo)"
  ],
  "resolve": null,
  "target_directory": "/foo/target",
  "version": 1,
  "workspace_root": "/foo"
}
"#;
    let meta: Metadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.packages.len(), 1);
    let pkg = &meta.packages[0];
    assert_eq!(pkg.name, "foo");
    assert_eq!(pkg.version, semver::Version::parse("0.1.0").unwrap());
    assert_eq!(pkg.authors.len(), 0);
    assert_eq!(pkg.id.to_string(), "foo 0.1.0 (path+file:///foo)");
    assert_eq!(pkg.description, None);
    assert_eq!(pkg.license, None);
    assert_eq!(pkg.license_file, None);
    assert_eq!(pkg.dependencies.len(), 1);
    let dep = &pkg.dependencies[0];
    assert_eq!(dep.name, "somedep");
    assert_eq!(dep.source, None);
    assert_eq!(dep.req, semver::VersionReq::parse("^1.0").unwrap());
    assert_eq!(dep.kind, cargo_metadata::DependencyKind::Normal);
    assert_eq!(dep.optional, false);
    assert_eq!(dep.uses_default_features, true);
    assert_eq!(dep.features.len(), 0);
    assert!(dep.target.is_none());
    assert_eq!(dep.rename, None);
    assert_eq!(dep.registry, None);
    assert_eq!(pkg.targets.len(), 1);
    let target = &pkg.targets[0];
    assert_eq!(target.name, "foo");
    assert_eq!(target.kind, vec!["bin"]);
    assert_eq!(target.crate_types, vec!["bin"]);
    assert_eq!(target.required_features.len(), 0);
    assert_eq!(target.src_path, PathBuf::from("/foo/src/main.rs"));
    assert_eq!(target.edition, "2015");
    assert_eq!(pkg.features.len(), 0);
    assert_eq!(pkg.manifest_path, PathBuf::from("/foo/Cargo.toml"));
    assert_eq!(pkg.categories.len(), 0);
    assert_eq!(pkg.keywords.len(), 0);
    assert_eq!(pkg.readme, None);
    assert_eq!(pkg.repository, None);
    assert_eq!(pkg.edition, "2015");
    assert_eq!(pkg.metadata, serde_json::Value::Null);
    assert_eq!(pkg.links, None);
    assert_eq!(meta.workspace_members.len(), 1);
    assert_eq!(
        meta.workspace_members[0].to_string(),
        "foo 0.1.0 (path+file:///foo)"
    );
    assert!(meta.resolve.is_none());
    assert_eq!(meta.workspace_root, PathBuf::from("/foo"));
    assert_eq!(meta.target_directory, PathBuf::from("/foo/target"));
}

macro_rules! sorted {
    ($e:expr) => {{
        let mut v = $e.clone();
        v.sort();
        v
    }};
}

fn cargo_version() -> semver::Version {
    let output = std::process::Command::new("cargo")
        .arg("-V")
        .output()
        .expect("Failed to exec cargo.");
    let out = std::str::from_utf8(&output.stdout).expect("invalid utf8").trim();
    let split: Vec<&str> = out.split_whitespace().collect();
    assert!(split.len() >= 2, "cargo -V output is unexpected: {}", out);
    semver::Version::parse(split[1]).expect("cargo -V semver could not be parsed")
}

#[test]
fn all_the_fields() {
    // All the fields currently generated as of 1.31. This tries to exercise as
    // much as possible.
    let ver = cargo_version();
    let minimum = semver::Version::parse("1.31.0").unwrap();
    if ver < minimum {
        // edition added in 1.30
        // rename added in 1.31
        eprintln!("Skipping all_the_fields test, cargo {} is too old.", ver);
        return;
    }
    let meta = MetadataCommand::new()
        .manifest_path("tests/all/Cargo.toml")
        .exec()
        .unwrap();
    assert_eq!(
        meta.workspace_root.file_name().unwrap(),
        PathBuf::from("all")
    );
    assert_eq!(
        meta.target_directory.file_name().unwrap(),
        PathBuf::from("target")
    );
    assert_eq!(meta.workspace_members.len(), 1);
    assert!(meta.workspace_members[0].to_string().starts_with("all"));

    assert_eq!(meta.packages.len(), 9);
    let all = meta.packages.iter().find(|p| p.name == "all").unwrap();
    assert_eq!(all.version, semver::Version::parse("0.1.0").unwrap());
    assert_eq!(all.authors, vec!["Jane Doe <user@example.com>"]);
    assert!(all.id.to_string().starts_with("all"));
    assert_eq!(all.description, Some("Package description.".to_string()));
    assert_eq!(all.license, Some("MIT/Apache-2.0".to_string()));
    assert_eq!(all.license_file, Some(PathBuf::from("LICENSE")));

    assert_eq!(all.dependencies.len(), 8);
    let bitflags = all
        .dependencies
        .iter()
        .find(|d| d.name == "bitflags")
        .unwrap();
    assert_eq!(
        bitflags.source,
        Some("registry+https://github.com/rust-lang/crates.io-index".to_string())
    );
    assert_eq!(bitflags.optional, true);
    assert_eq!(bitflags.req, semver::VersionReq::parse("^1.0").unwrap());

    let path_dep = all
        .dependencies
        .iter()
        .find(|d| d.name == "path-dep")
        .unwrap();
    assert_eq!(path_dep.source, None);
    assert_eq!(path_dep.kind, cargo_metadata::DependencyKind::Normal);
    assert_eq!(path_dep.req, semver::VersionReq::parse("*").unwrap());

    all.dependencies
        .iter()
        .find(|d| d.name == "namedep")
        .unwrap();

    let featdep = all
        .dependencies
        .iter()
        .find(|d| d.name == "featdep")
        .unwrap();
    assert_eq!(featdep.features, vec!["i128"]);
    assert_eq!(featdep.uses_default_features, false);

    let renamed = all
        .dependencies
        .iter()
        .find(|d| d.name == "oldname")
        .unwrap();
    assert_eq!(renamed.rename, Some("newname".to_string()));

    let devdep = all
        .dependencies
        .iter()
        .find(|d| d.name == "devdep")
        .unwrap();
    assert_eq!(devdep.kind, cargo_metadata::DependencyKind::Development);

    let bdep = all.dependencies.iter().find(|d| d.name == "bdep").unwrap();
    assert_eq!(bdep.kind, cargo_metadata::DependencyKind::Build);

    let windep = all
        .dependencies
        .iter()
        .find(|d| d.name == "windep")
        .unwrap();
    assert_eq!(
        windep.target.as_ref().map(|x| x.to_string()),
        Some("cfg(windows)".to_string())
    );

    macro_rules! get_file_name {
        ($v:expr) => {
            all.targets
                .iter()
                .find(|t| t.src_path.file_name().unwrap() == $v)
                .unwrap()
        };
    }
    assert_eq!(all.targets.len(), 8);
    let lib = get_file_name!("lib.rs");
    assert_eq!(lib.name, "all");
    assert_eq!(
        sorted!(lib.kind),
        vec!["cdylib", "dylib", "rlib", "staticlib"]
    );
    assert_eq!(
        sorted!(lib.crate_types),
        vec!["cdylib", "dylib", "rlib", "staticlib"]
    );
    assert_eq!(lib.required_features.len(), 0);
    assert_eq!(lib.edition, "2018");

    let main = get_file_name!("main.rs");
    assert_eq!(main.crate_types, vec!["bin"]);
    assert_eq!(main.kind, vec!["bin"]);

    let otherbin = get_file_name!("otherbin.rs");
    assert_eq!(otherbin.edition, "2015");

    let reqfeat = get_file_name!("reqfeat.rs");
    assert_eq!(reqfeat.required_features, vec!["feat2"]);

    let ex1 = get_file_name!("ex1.rs");
    assert_eq!(ex1.kind, vec!["example"]);

    let t1 = get_file_name!("t1.rs");
    assert_eq!(t1.kind, vec!["test"]);

    let b1 = get_file_name!("b1.rs");
    assert_eq!(b1.kind, vec!["bench"]);

    let build = get_file_name!("build.rs");
    assert_eq!(build.kind, vec!["custom-build"]);

    assert_eq!(all.features.len(), 3);
    assert_eq!(all.features["feat1"].len(), 0);
    assert_eq!(all.features["feat2"].len(), 0);
    assert_eq!(sorted!(all.features["default"]), vec!["bitflags", "feat1"]);

    assert!(all.manifest_path.ends_with("all/Cargo.toml"));
    assert_eq!(all.categories, vec!["command-line-utilities"]);
    assert_eq!(all.keywords, vec!["cli"]);
    assert_eq!(all.readme, Some("README.md".to_string()));
    assert_eq!(
        all.repository,
        Some("https://github.com/oli-obk/cargo_metadata/".to_string())
    );
    assert_eq!(all.edition, "2018");
    assert_eq!(
        all.metadata,
        json!({
            "docs": {
                "rs": {
                    "all-features": true,
                    "default-target": "x86_64-unknown-linux-gnu",
                    "rustc-args": ["--example-rustc-arg"]
                }
            }
        })
    );

    let resolve = meta.resolve.as_ref().unwrap();
    assert!(resolve
        .root
        .as_ref()
        .unwrap()
        .to_string()
        .starts_with("all"));

    assert_eq!(resolve.nodes.len(), 9);
    let path_dep = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("path-dep"))
        .unwrap();
    assert_eq!(path_dep.deps.len(), 0);
    assert_eq!(path_dep.dependencies.len(), 0);
    assert_eq!(path_dep.features.len(), 0);

    let bitflags = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("bitflags"))
        .unwrap();
    assert_eq!(bitflags.features, vec!["default"]);

    let featdep = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("featdep"))
        .unwrap();
    assert_eq!(featdep.features, vec!["i128"]);

    let all = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("all"))
        .unwrap();
    assert_eq!(all.dependencies.len(), 8);
    assert_eq!(all.deps.len(), 8);
    let newname = all.deps.iter().find(|d| d.name == "newname").unwrap();
    assert!(newname.pkg.to_string().starts_with("oldname"));
    // Note the underscore here.
    let path_dep = all.deps.iter().find(|d| d.name == "path_dep").unwrap();
    assert!(path_dep.pkg.to_string().starts_with("path-dep"));
    let namedep = all
        .deps
        .iter()
        .find(|d| d.name == "different_name")
        .unwrap();
    assert!(namedep.pkg.to_string().starts_with("namedep"));
    assert_eq!(sorted!(all.features), vec!["bitflags", "default", "feat1"]);
}

#[test]
fn alt_registry() {
    // This is difficult to test (would need to set up a custom index).
    // Just manually check the JSON is handled.
    let json = r#"
{
  "packages": [
    {
      "name": "alt",
      "version": "0.1.0",
      "id": "alt 0.1.0 (path+file:///alt)",
      "source": null,
      "dependencies": [
        {
          "name": "alt2",
          "source": "registry+https://example.com",
          "req": "^0.1",
          "kind": null,
          "rename": null,
          "optional": false,
          "uses_default_features": true,
          "features": [],
          "target": null,
          "registry": "https://example.com"
        }
      ],
      "targets": [
        {
          "kind": [
            "lib"
          ],
          "crate_types": [
            "lib"
          ],
          "name": "alt",
          "src_path": "/alt/src/lib.rs",
          "edition": "2018"
        }
      ],
      "features": {},
      "manifest_path": "/alt/Cargo.toml",
      "metadata": null,
      "authors": [],
      "categories": [],
      "keywords": [],
      "readme": null,
      "repository": null,
      "edition": "2018",
      "links": null
    }
  ],
  "workspace_members": [
    "alt 0.1.0 (path+file:///alt)"
  ],
  "resolve": null,
  "target_directory": "/alt/target",
  "version": 1,
  "workspace_root": "/alt"
}
"#;
    let meta: Metadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.packages.len(), 1);
    let alt = &meta.packages[0];
    let deps = &alt.dependencies;
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    assert_eq!(dep.registry, Some("https://example.com".to_string()));
}

#[test]
fn links() {
    // This should be moved to all_the_fields once it is available on stable (1.33).
    let json = r#"
{
  "packages": [
    {
      "name": "foo",
      "version": "0.1.0",
      "id": "foo 0.1.0 (path+file:///foo)",
      "license": null,
      "license_file": null,
      "description": null,
      "source": null,
      "dependencies": [],
      "targets": [
        {
          "kind": [
            "lib"
          ],
          "crate_types": [
            "lib"
          ],
          "name": "foo",
          "src_path": "/foo/src/lib.rs",
          "edition": "2018"
        }
      ],
      "features": {},
      "manifest_path": "/foo/Cargo.toml",
      "metadata": null,
      "authors": [],
      "categories": [],
      "keywords": [],
      "readme": null,
      "repository": null,
      "edition": "2018",
      "links": "somelib"
    }
  ],
  "workspace_members": [
    "foo 0.1.0 (path+file:///foo)"
  ],
  "resolve": null,
  "target_directory": "/foo/target",
  "version": 1,
  "workspace_root": "/foo"
}
"#;
    let meta: Metadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.packages[0].links, Some("somelib".to_string()));
}


#[test]
fn current_dir() {
    let meta = MetadataCommand::new()
        .current_dir("tests/all/namedep")
        .exec()
        .unwrap();
    let namedep = meta.packages.iter().find(|p| p.name == "namedep").unwrap();
    assert!(namedep.name.starts_with("namedep"));
}
