# Changelog

## Unreleased

## 0.13.1

- OpenAPI specification links in docs now reference the authoritative HTML version.

## 0.13.0

- Add `spec::ObjectSchema::deprecated` field.
- Add `spec::ObjectSchema::examples` field.
- Add `spec::Contact::validate_email()` method.
- Add `spec::Discriminator` type.
- Add `spec::ObjectSchema::discriminator` field.
- Expose the `spec::ClientCredentialsFlow::token_url` field.
- The type of the `spec::ObjectSchema::enum` field is now `Vec<serde_json::Value>`.
- The type of the `spec::ObjectSchema::const` field is now `Option<serde_json::Value>`.
- Rename `Error::{SemVerError => Semver}` enum variant.
- Rename `spec::RefError::{InvalidType => UnknownType}` enum variant.

## 0.12.1

- No significant changes since `0.12.0`.

## 0.12.0

- Completely re-work `spec::Header`, updating it to conform to the OpenAPI v3.1 spec.
- Allow explicit `null` schema examples to be deserialized as `Some(serde_json::Value::Null)`.
- Remove feature guard on `spec::Typeset::{is_object_or_nullable_object, is_array_or_nullable_array}()` methods.
- Remove `validation` feature. (Functionality migrated to `roast` crate).
- Remove `conformance` feature. (Functionality migrated to `roast` crate).

## 0.11.0

- Add `spec::Schema::const_value` field.
- Add `spec::ObjectSchema::extensions` field.

## 0.10.0

- Add `spec::Info::extensions` field.
- Remove top-level `ObjectSchema` re-export.

## 0.9.0

- Rename `spec::{Schema => ObjectSchema}` struct.
- Add `spec::BooleanSchema` struct.
- Add `spec::Schema` enum.
- The `spec::ObjectSchema::addition_properties` field is now of type `Option<Schema>`.
- The `spec::Parameter::schema` field is now of type `ObjectOrReference<ObjectSchema>`.
- Add `spec::Operation::extensions` field.
- Minimum supported Rust version (MSRV) is now 1.75.

## 0.8.1

- Fix `spec::Parameter` deserialization when no `examples` are present.

## 0.8.0

- Add `spec::Parameter::{example, examples, content}` fields.
- Implement `FromRef` for `spec::Header`.
- Remove `spec::Parameter::unique_items` field.

## 0.7.0

- Add `Spec::extensions` field.
- Add `spec::{Components, Contact, Example, ExternalDoc, License, Link::{Id, Ref}, Parameter, PathItem, RequestBody, Response, Tag}::extensions` fields.
- Add `spec::ParameterIn` enum.
- Add `spec::ParameterStyle::{Matrix, Label, SpaceDelimited, PipeDelimited, DeepObject}` variants.
- The `spec::Parameter::location` field is now of type `ParameterIn`.
- Narrow version range allowed by `Spec::validate_version()` to `~3.1`.
- Remove `Default` implementation for `Spec`.
- Remove `Default` implementation for `spec::{Info, License, Parameter, Server, ServerVariable}`.

## 0.6.0

- Add `from_str()` function.
- Add `spec::SecurityScheme::MutualTls` enum variant.
- Add `spec::SecurityScheme::{ApiKey, Http, OAuth2, OpenIdConnect}::description` fields.
- Add `spec::SchemaTypeSet` enum.
- Add `spec::SchemaType::Null` variant.
- The `Spec::paths` field is now optional to closer align with the spec.
- The `spec::Operation::responses` field is now optional to closer align with the spec.
- The `spec::Schema::{exclusive_maximum, exclusive_minimum}` fields are now of type `Option<serde_json:Number>` to closer align with the spec.
- Migrate YAML parsing to `serde_yml`. Exposed error type(s) have been altered.
- Add `spec::Schema::nullable` field.

## 0.5.0

- Update `http` dependency to `1`.

## 0.4.0

- The `SecurityScheme::Http::bearer_format` is now optional.

## 0.3.0

- Initial re-release.

## 0.2.1

- Last version derived from <https://github.com/softprops/openapi>.
