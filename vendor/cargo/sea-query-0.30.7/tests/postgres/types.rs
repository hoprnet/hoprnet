use super::*;
use pretty_assertions::assert_eq;
use sea_query::{extension::postgres::Type, Alias, PostgresQueryBuilder};

#[test]
fn create_1() {
    assert_eq!(
        Type::create()
            .as_enum(Font::Table)
            .values([Font::Name, Font::Variant, Font::Language])
            .to_string(PostgresQueryBuilder),
        r#"CREATE TYPE "font" AS ENUM ('name', 'variant', 'language')"#
    );
}

#[test]
fn create_2() {
    assert_eq!(
        Type::create()
            .as_enum((Alias::new("schema"), Font::Table))
            .values([Font::Name, Font::Variant, Font::Language])
            .to_string(PostgresQueryBuilder),
        r#"CREATE TYPE "schema"."font" AS ENUM ('name', 'variant', 'language')"#
    );
}

#[test]
fn drop_1() {
    assert_eq!(
        Type::drop()
            .if_exists()
            .name(Font::Table)
            .restrict()
            .to_string(PostgresQueryBuilder),
        r#"DROP TYPE IF EXISTS "font" RESTRICT"#
    )
}

#[test]
fn drop_2() {
    assert_eq!(
        Type::drop()
            .name(Font::Table)
            .to_string(PostgresQueryBuilder),
        r#"DROP TYPE "font""#
    );
}

#[test]
fn drop_3() {
    assert_eq!(
        Type::drop()
            .if_exists()
            .name(Font::Table)
            .cascade()
            .to_string(PostgresQueryBuilder),
        r#"DROP TYPE IF EXISTS "font" CASCADE"#
    );
}

#[test]
fn drop_4() {
    assert_eq!(
        Type::drop()
            .name((Alias::new("schema"), Font::Table))
            .to_string(PostgresQueryBuilder),
        r#"DROP TYPE "schema"."font""#
    );
}

#[test]
fn alter_1() {
    assert_eq!(
        Type::alter()
            .name(Font::Table)
            .add_value(Alias::new("weight"))
            .to_string(PostgresQueryBuilder),
        r#"ALTER TYPE "font" ADD VALUE 'weight'"#
    )
}
#[test]
fn alter_2() {
    assert_eq!(
        Type::alter()
            .name(Font::Table)
            .add_value(Alias::new("weight"))
            .before(Font::Variant)
            .to_string(PostgresQueryBuilder),
        r#"ALTER TYPE "font" ADD VALUE 'weight' BEFORE 'variant'"#
    )
}

#[test]
fn alter_3() {
    assert_eq!(
        Type::alter()
            .name(Font::Table)
            .add_value(Alias::new("weight"))
            .after(Font::Variant)
            .to_string(PostgresQueryBuilder),
        r#"ALTER TYPE "font" ADD VALUE 'weight' AFTER 'variant'"#
    )
}

#[test]
fn alter_4() {
    assert_eq!(
        Type::alter()
            .name(Font::Table)
            .rename_to(Alias::new("typeface"))
            .to_string(PostgresQueryBuilder),
        r#"ALTER TYPE "font" RENAME TO 'typeface'"#
    )
}

#[test]
fn alter_5() {
    assert_eq!(
        Type::alter()
            .name(Font::Table)
            .rename_value(Font::Variant, Font::Language)
            .to_string(PostgresQueryBuilder),
        r#"ALTER TYPE "font" RENAME VALUE 'variant' TO 'language'"#
    )
}

#[test]
fn alter_6() {
    assert_eq!(
        Type::alter()
            .name((Alias::new("schema"), Font::Table))
            .rename_to(Alias::new("typeface"))
            .to_string(PostgresQueryBuilder),
        r#"ALTER TYPE "schema"."font" RENAME TO 'typeface'"#
    )
}
