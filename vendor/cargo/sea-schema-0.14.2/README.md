<div align="center">

  <img src="docs/SeaQL logo dual.png" width="320"/>

  <h1>SeaSchema</h1>

  <p>
    <strong>🌿 SQL schema definition and discovery</strong>
  </p>

  [![crate](https://img.shields.io/crates/v/sea-schema.svg)](https://crates.io/crates/sea-schema)
  [![docs](https://docs.rs/sea-schema/badge.svg)](https://docs.rs/sea-schema)
  [![build status](https://github.com/SeaQL/sea-schema/actions/workflows/rust.yml/badge.svg)](https://github.com/SeaQL/sea-schema/actions/workflows/rust.yml)

</div>

## About

SeaSchema is a library to help you manage database schema for MySQL, Postgres and SQLite. It provides 1) type definitions for representing database schema mapping each database closely and 2) utilities to discover them.

[![GitHub stars](https://img.shields.io/github/stars/SeaQL/sea-schema.svg?style=social&label=Star&maxAge=1)](https://github.com/SeaQL/sea-schema/stargazers/)
If you like what we do, consider starring, commenting, sharing and contributing!

[![Discord](https://img.shields.io/discord/873880840487206962?label=Discord)](https://discord.com/invite/uCPdDXzbdv)
Join our Discord server to chat with others in the SeaQL community!

## Architecture

The crate is divided into different modules:

+ `def`: type definitions
+ `query`, `parser`: for querying and parsing information_schema
+ `discovery`: connect to a live database and discover a `Schema`
+ `writer`: for exporting `Schema` into SeaQuery and SQL statements

JSON de/serialize on type definitions can be enabled with `with-serde`.

## Schema Discovery

Take the MySQL [Sakila Sample Database](tests/sakila/mysql/sakila-schema.sql) as example, given the following table:

```SQL
CREATE TABLE film_actor (
  actor_id SMALLINT UNSIGNED NOT NULL,
  film_id SMALLINT UNSIGNED NOT NULL,
  last_update TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY  (actor_id,film_id),
  KEY idx_fk_film_id (`film_id`),
  CONSTRAINT fk_film_actor_actor FOREIGN KEY (actor_id) REFERENCES actor (actor_id) ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT fk_film_actor_film FOREIGN KEY (film_id) REFERENCES film (film_id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

```

The [discovered schema result](tests/discovery/mysql/schema.rs):

```rust
TableDef {
    info: TableInfo {
        name: "film_actor",
        engine: InnoDb,
        auto_increment: None,
        char_set: Utf8Mb4,
        collation: Utf8Mb40900AiCi,
        comment: "",
    },
    columns: [
        ColumnInfo {
            name: "actor_id",
            col_type: SmallInt(
                NumericAttr {
                    maximum: None,
                    decimal: None,
                    unsigned: Some(true),
                    zero_fill: None,
                },
            ),
            null: false,
            key: Primary,
            default: None,
            extra: ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: false,
            },
            expression: None,
            comment: "",
        },
        ColumnInfo {
            name: "film_id",
            col_type: SmallInt(
                NumericAttr {
                    maximum: None,
                    decimal: None,
                    unsigned: Some(true),
                    zero_fill: None,
                },
            ),
            null: false,
            key: Primary,
            default: None,
            extra: ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: false,
            },
            expression: None,
            comment: "",
        },
        ColumnInfo {
            name: "last_update",
            col_type: Timestamp(TimeAttr { fractional: None }),
            null: false,
            key: NotKey,
            default: Some(ColumnDefault::CurrentTimestamp),
            extra: ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: true,
                generated: false,
                default_generated: true,
            },
            expression: None,
            comment: "",
        },
    ],
    indexes: [
        IndexInfo {
            unique: false,
            name: "idx_fk_film_id",
            parts: [
                IndexPart {
                    column: "film_id",
                    order: Ascending,
                    sub_part: None,
                },
            ],
            nullable: false,
            idx_type: BTree,
            comment: "",
            functional: false,
        },
        IndexInfo {
            unique: true,
            name: "PRIMARY",
            parts: [
                IndexPart {
                    column: "actor_id",
                    order: Ascending,
                    sub_part: None,
                },
                IndexPart {
                    column: "film_id",
                    order: Ascending,
                    sub_part: None,
                },
            ],
            nullable: false,
            idx_type: BTree,
            comment: "",
            functional: false,
        },
    ],
    foreign_keys: [
        ForeignKeyInfo {
            name: "fk_film_actor_actor",
            columns: [ "actor_id" ],
            referenced_table: "actor",
            referenced_columns: [ "actor_id" ],
            on_update: Cascade,
            on_delete: Restrict,
        },
        ForeignKeyInfo {
            name: "fk_film_actor_film",
            columns: [ "film_id" ],
            referenced_table: "film",
            referenced_columns: [ "film_id" ],
            on_update: Cascade,
            on_delete: Restrict,
        },
    ],
}
```

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

SeaSchema is a community driven project. We welcome you to participate, contribute and together build for Rust's future.

A big shout out to our contributors:

[![Contributors](https://opencollective.com/sea-schema/contributors.svg?width=1000&button=false)](https://github.com/SeaQL/sea-schema/graphs/contributors)
