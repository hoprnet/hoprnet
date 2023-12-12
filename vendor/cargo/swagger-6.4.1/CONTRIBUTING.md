# Contributing to `swagger-rs`

Thanks for your interest - we gratefully welcome contributions.

Questions can be asked in [issues](https://github.com/Metaswitch/swagger-rs/issues).

To help us help you get pull requests merged quickly and smoothly, open an issue before submitted large changes. Please keep the contents of pull requests and commits short. Commit messages should include the intent of the commit. Pull requests should add an entry to [CHANGELOG.md].

Contributions that add/improve tests are awesome. Please add tests for every change.

`swagger-rs` uses [`rustfmt`](https://github.com/rust-lang-nursery/rustfmt) for formatting and [`clippy`](https://github.com/rust-lang-nursery/rust-clippy) for linting. See .travis.yml for the versions we use.

## Testing against openapi-generator

If you are making a non-trivial change, please open a simultaneous PR against openapi-generator. This will allow the openapi-generator CI pipeline to verify that your change is compatible.

You will need to update `modules/openapi-generator/src/main/resources/rust-server/Cargo.mustache` as follows:

```toml
swagger = { git = "https://github.com/foo/swagger-rs.git", branch = "bar"}
```

## Conduct

In all `swagger-rs`-related forums, we follow the [Swagger Codegen Code of Conduct](https://github.com/swagger-api/swagger-codegen/blob/master/CODE_OF_CONDUCT.md). For escalation or moderation issues please contact Benjamin (mailto:benjamin.gill@metaswitch.com) instead of the swagger-api moderation team.
