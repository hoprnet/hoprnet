use crate::mysql::def::*;
use crate::mysql::query::ColumnQueryResult;
use crate::{parser::Parser, Name};
use sea_query::{EscapeBuilder, MysqlQueryBuilder};

impl ColumnQueryResult {
    pub fn parse(self, system: &SystemInfo) -> ColumnInfo {
        parse_column_query_result(self, system)
    }
}

pub fn parse_column_query_result(result: ColumnQueryResult, system: &SystemInfo) -> ColumnInfo {
    let col_type = parse_column_type(&mut Parser::new(&result.column_type));
    let default = parse_column_default(&col_type, result.column_default, &result.extra, system);
    ColumnInfo {
        name: result.column_name,
        col_type,
        null: parse_column_null(&result.is_nullable),
        key: parse_column_key(&result.column_key),
        default,
        extra: parse_column_extra(&mut Parser::new(&result.extra)),
        expression: match result.generation_expression {
            Some(generation_expression) => parse_generation_expression(generation_expression),
            None => None,
        },
        comment: result.column_comment,
    }
}

pub fn parse_column_type(parser: &mut Parser) -> ColumnType {
    let mut type_name = "";
    if parser.curr().is_none() {
        return Type::Unknown(type_name.to_owned());
    }
    if let Some(word) = parser.next_if_unquoted_any() {
        type_name = word.as_str();
    }
    let ctype = parse_type_name(type_name);
    if ctype.is_numeric() {
        parse_numeric_attributes(parser, ctype)
    } else if ctype.is_time() {
        parse_time_attributes(parser, ctype)
    } else if ctype.is_string() {
        parse_string_attributes(parser, ctype)
    } else if ctype.is_free_size_blob() {
        parse_blob_attributes(parser, ctype)
    } else if ctype.is_enum() {
        parse_enum_definition(parser, ctype)
    } else if ctype.is_set() {
        parse_set_definition(parser, ctype)
    } else if ctype.is_geometry() {
        parse_geometry_attributes(parser, ctype)
    } else {
        ctype
    }
}

pub fn parse_type_name(type_name: &str) -> Type {
    match type_name.to_lowercase().as_str() {
        "serial" => Type::Serial,
        "bit" => Type::Bit(NumericAttr::default()),
        "tinyint" => Type::TinyInt(NumericAttr::default()),
        "bool" => Type::Bool,
        "smallint" => Type::SmallInt(NumericAttr::default()),
        "mediumint" => Type::MediumInt(NumericAttr::default()),
        "int" => Type::Int(NumericAttr::default()),
        "integer" => Type::Int(NumericAttr::default()),
        "bigint" => Type::BigInt(NumericAttr::default()),
        "decimal" => Type::Decimal(NumericAttr::default()),
        "dec" => Type::Decimal(NumericAttr::default()),
        "fixed" => Type::Decimal(NumericAttr::default()),
        "float" => Type::Float(NumericAttr::default()),
        "double" => Type::Double(NumericAttr::default()),
        "date" => Type::Date,
        "time" => Type::Time(TimeAttr::default()),
        "datetime" => Type::DateTime(TimeAttr::default()),
        "timestamp" => Type::Timestamp(TimeAttr::default()),
        "year" => Type::Year,
        "char" => Type::Char(StringAttr::default()),
        "nchar" => Type::NChar(StringAttr::default()),
        "varchar" => Type::Varchar(StringAttr::default()),
        "nvarchar" => Type::NVarchar(StringAttr::default()),
        "binary" => Type::Binary(StringAttr::default()),
        "varbinary" => Type::Varbinary(StringAttr::default()),
        "text" => Type::Text(StringAttr::default()),
        "tinytext" => Type::TinyText(StringAttr::default()),
        "mediumtext" => Type::MediumText(StringAttr::default()),
        "longtext" => Type::LongText(StringAttr::default()),
        "blob" => Type::Blob(BlobAttr::default()),
        "tinyblob" => Type::TinyBlob,
        "mediumblob" => Type::MediumBlob,
        "longblob" => Type::LongBlob,
        "enum" => Type::Enum(EnumDef::default()),
        "set" => Type::Set(SetDef::default()),
        "geometry" => Type::Geometry(GeometryAttr::default()),
        "point" => Type::Point(GeometryAttr::default()),
        "linestring" => Type::LineString(GeometryAttr::default()),
        "polygon" => Type::Polygon(GeometryAttr::default()),
        "multipoint" => Type::MultiPoint(GeometryAttr::default()),
        "multilinestring" => Type::MultiLineString(GeometryAttr::default()),
        "multipolygon" => Type::MultiPolygon(GeometryAttr::default()),
        "geometrycollection" => Type::GeometryCollection(GeometryAttr::default()),
        "json" => Type::Json,
        _ => Type::Unknown(type_name.to_owned()),
    }
}

fn parse_numeric_attributes(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_punctuation("(") {
        if let Some(word) = parser.next_if_unquoted_any() {
            if let Ok(number) = word.as_str().parse::<u32>() {
                ctype.get_numeric_attr_mut().maximum = Some(number);
            }
        }

        if parser.next_if_punctuation(",") {
            if let Some(word) = parser.next_if_unquoted_any() {
                if let Ok(number) = word.as_str().parse::<u32>() {
                    ctype.get_numeric_attr_mut().decimal = Some(number);
                }
            }
        }

        parser.next_if_punctuation(")");
    }

    if parser.next_if_unquoted("unsigned") {
        ctype.get_numeric_attr_mut().unsigned = Some(true);
    }

    if parser.next_if_unquoted("zerofill") {
        ctype.get_numeric_attr_mut().zero_fill = Some(true);
    }

    ctype
}

fn parse_time_attributes(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_punctuation("(") {
        if let Some(word) = parser.next_if_unquoted_any() {
            if let Ok(number) = word.as_str().parse::<u32>() {
                ctype.get_time_attr_mut().fractional = Some(number);
            }
        }
        parser.next_if_punctuation(")");
    }

    ctype
}

fn parse_string_attributes(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_punctuation("(") {
        if let Some(word) = parser.next_if_unquoted_any() {
            if let Ok(number) = word.as_str().parse::<u32>() {
                ctype.get_string_attr_mut().length = Some(number);
            }
        }
        parser.next_if_punctuation(")");
    }

    parse_charset_collate(parser, ctype.get_string_attr_mut());

    ctype
}

fn parse_charset_collate(parser: &mut Parser, str_attr: &mut StringAttr) {
    if parser.next_if_unquoted("character") && parser.next_if_unquoted("set") {
        if let Some(word) = parser.next_if_unquoted_any() {
            str_attr.charset = CharSet::from_str(word.as_str());
        }
    }

    if parser.next_if_unquoted("collate") {
        if let Some(word) = parser.next_if_unquoted_any() {
            str_attr.collation = Collation::from_str(word.as_str());
        }
    }
}

fn parse_blob_attributes(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_punctuation("(") {
        if let Some(word) = parser.next_if_unquoted_any() {
            if let Ok(number) = word.as_str().parse::<u32>() {
                ctype.get_blob_attr_mut().length = Some(number);
            }
        }
        parser.next_if_punctuation(")");
    }

    ctype
}

fn parse_enum_definition(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_punctuation("(") {
        while parser.curr().is_some() {
            if let Some(word) = parser.next_if_quoted_any() {
                ctype
                    .get_enum_def_mut()
                    .values
                    .push(MysqlQueryBuilder.unescape_string(word.unquote().unwrap().as_str()));
                parser.next_if_punctuation(",");
            } else if parser.curr_is_unquoted() {
                todo!("there can actually be numeric enum values but is very confusing");
            }
            if parser.next_if_punctuation(")") {
                break;
            }
        }
    }

    parse_charset_collate(parser, &mut ctype.get_enum_def_mut().attr);

    ctype
}

fn parse_set_definition(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_punctuation("(") {
        while parser.curr().is_some() {
            if let Some(word) = parser.next_if_quoted_any() {
                ctype
                    .get_set_def_mut()
                    .members
                    .push(MysqlQueryBuilder.unescape_string(word.unquote().unwrap().as_str()));
                parser.next_if_punctuation(",");
            } else if parser.curr_is_unquoted() {
                todo!("there can actually be numeric set values but is very confusing");
            }
            if parser.next_if_punctuation(")") {
                break;
            }
        }
    }

    parse_charset_collate(parser, &mut ctype.get_set_def_mut().attr);

    ctype
}

fn parse_geometry_attributes(parser: &mut Parser, mut ctype: ColumnType) -> ColumnType {
    if parser.next_if_unquoted("srid") {
        if let Some(word) = parser.next_if_unquoted_any() {
            if let Ok(number) = word.as_str().parse::<u32>() {
                ctype.get_geometry_attr_mut().srid = Some(number);
            }
        }
        parser.next_if_punctuation(")");
    }

    ctype
}

pub fn parse_column_null(string: &str) -> bool {
    matches!(string.to_uppercase().as_str(), "YES")
}

pub fn parse_column_key(string: &str) -> ColumnKey {
    match string.to_uppercase().as_str() {
        "PRI" => ColumnKey::Primary,
        "UNI" => ColumnKey::Unique,
        "MUL" => ColumnKey::Multiple,
        _ => ColumnKey::NotKey,
    }
}

pub fn parse_column_default(
    col_type: &Type,
    default: Option<String>,
    extra: &str,
    system: &SystemInfo,
) -> Option<ColumnDefault> {
    match default {
        Some(default) => {
            if !default.is_empty() {
                let default_value = if system.is_mysql() && system.version >= 80000 {
                    parse_mysql_8_default(default, extra)
                } else if system.is_maria_db() && system.version >= 100207 {
                    parse_mariadb_10_default(default)
                } else {
                    parse_mysql_5_default(default, col_type)
                };
                Some(default_value)
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn parse_mysql_5_default(default: String, col_type: &Type) -> ColumnDefault {
    let is_date_time = matches!(col_type, Type::DateTime(_) | Type::Timestamp(_));
    if is_date_time && default == "CURRENT_TIMESTAMP" {
        ColumnDefault::CurrentTimestamp
    } else if let Ok(int) = default.parse() {
        ColumnDefault::Int(int)
    } else if let Ok(real) = default.parse() {
        ColumnDefault::Real(real)
    } else {
        ColumnDefault::String(default)
    }
}

pub fn parse_mysql_8_default(default: String, extra: &str) -> ColumnDefault {
    let is_expression = extra.contains("DEFAULT_GENERATED");
    if is_expression && default == "CURRENT_TIMESTAMP" {
        ColumnDefault::CurrentTimestamp
    } else if is_expression && default == "NULL" {
        ColumnDefault::Null
    } else if let Ok(int) = default.parse() {
        ColumnDefault::Int(int)
    } else if let Ok(real) = default.parse() {
        ColumnDefault::Real(real)
    } else if is_expression {
        ColumnDefault::CustomExpr(default)
    } else {
        ColumnDefault::String(default)
    }
}

pub fn parse_mariadb_10_default(default: String) -> ColumnDefault {
    if default.starts_with('\'') && default.ends_with('\'') {
        ColumnDefault::String(default[1..(default.len() - 1)].into())
    } else if let Ok(int) = default.parse() {
        ColumnDefault::Int(int)
    } else if let Ok(real) = default.parse() {
        ColumnDefault::Real(real)
    } else if default == "current_timestamp()" {
        ColumnDefault::CurrentTimestamp
    } else if default == "NULL" {
        ColumnDefault::Null
    } else {
        ColumnDefault::CustomExpr(default)
    }
}

pub fn parse_generation_expression(string: String) -> Option<ColumnExpression> {
    if string.is_empty() {
        None
    } else {
        Some(ColumnExpression { expr: string })
    }
}

pub fn parse_column_extra(parser: &mut Parser) -> ColumnExtra {
    let mut extra = ColumnExtra::default();

    while parser.curr().is_some() {
        // order does not matter
        if parser.next_if_unquoted("on") {
            if parser.next_if_unquoted("update") && parser.next_if_unquoted("current_timestamp") {
                extra.on_update_current_timestamp = true;
            }
        } else if parser.next_if_unquoted("auto_increment") {
            extra.auto_increment = true;
        } else if parser.next_if_unquoted("default_generated") {
            extra.default_generated = true;
        } else if parser.next_if_unquoted("stored") || parser.next_if_unquoted("virtual") {
            if parser.next_if_unquoted("generated") {
                extra.generated = true;
            }
        } else {
            parser.next();
        }
    }

    extra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0() {
        assert_eq!(
            parse_column_extra(&mut Parser::new("")),
            ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: false,
            }
        );
    }

    #[test]
    fn test_0b() {
        assert_eq!(
            parse_column_extra(&mut Parser::new("NOTHING matters")),
            ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: false,
            }
        );
    }

    #[test]
    fn test_1() {
        assert_eq!(
            parse_column_extra(&mut Parser::new("DEFAULT_GENERATED")),
            ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: true,
            }
        );
    }

    #[test]
    fn test_1b() {
        assert_eq!(
            parse_column_extra(&mut Parser::new("DEFAULT_GENERATED garbage")),
            ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: true,
            }
        );
    }

    #[test]
    fn test_1c() {
        assert_eq!(
            parse_column_extra(&mut Parser::new("garbage DEFAULT_GENERATED")),
            ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: false,
                generated: false,
                default_generated: true,
            }
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(
            parse_column_extra(&mut Parser::new(
                "DEFAULT_GENERATED on update CURRENT_TIMESTAMP"
            )),
            ColumnExtra {
                auto_increment: false,
                on_update_current_timestamp: true,
                generated: false,
                default_generated: true,
            }
        );
    }

    #[test]
    fn test_3() {
        assert_eq!(
            parse_column_type(&mut Parser::new("smallint unsigned")),
            ColumnType::SmallInt(NumericAttr {
                maximum: None,
                decimal: None,
                unsigned: Some(true),
                zero_fill: None,
            })
        );
    }

    #[test]
    fn test_4() {
        assert_eq!(
            parse_column_type(&mut Parser::new("smallint unsigned zerofill")),
            ColumnType::SmallInt(NumericAttr {
                maximum: None,
                decimal: None,
                unsigned: Some(true),
                zero_fill: Some(true),
            })
        );
    }

    #[test]
    fn test_5() {
        assert_eq!(
            parse_column_type(&mut Parser::new("decimal(4,2)")),
            ColumnType::Decimal(NumericAttr {
                maximum: Some(4),
                decimal: Some(2),
                unsigned: None,
                zero_fill: None,
            })
        );
    }

    #[test]
    fn test_6() {
        assert_eq!(
            parse_column_type(&mut Parser::new("decimal(18,4) zerofill")),
            ColumnType::Decimal(NumericAttr {
                maximum: Some(18),
                decimal: Some(4),
                unsigned: None,
                zero_fill: Some(true),
            })
        );
    }

    #[test]
    fn test_7() {
        assert_eq!(
            parse_column_type(&mut Parser::new("decimal(18,4) unsigned")),
            ColumnType::Decimal(NumericAttr {
                maximum: Some(18),
                decimal: Some(4),
                unsigned: Some(true),
                zero_fill: None,
            })
        );
    }

    #[test]
    fn test_8() {
        assert_eq!(
            parse_column_type(&mut Parser::new("decimal(18,4) unsigned zerofill")),
            ColumnType::Decimal(NumericAttr {
                maximum: Some(18),
                decimal: Some(4),
                unsigned: Some(true),
                zero_fill: Some(true),
            })
        );
    }

    #[test]
    fn test_9() {
        assert_eq!(
            parse_column_type(&mut Parser::new("smallint(8) unsigned zerofill")),
            ColumnType::SmallInt(NumericAttr {
                maximum: Some(8),
                decimal: None,
                unsigned: Some(true),
                zero_fill: Some(true),
            })
        );
    }

    #[test]
    fn test_10() {
        assert_eq!(
            parse_column_type(&mut Parser::new("DATETIME")),
            ColumnType::DateTime(TimeAttr { fractional: None })
        );
    }

    #[test]
    fn test_11() {
        assert_eq!(
            parse_column_type(&mut Parser::new("DATETIME(6)")),
            ColumnType::DateTime(TimeAttr {
                fractional: Some(6),
            })
        );
    }

    #[test]
    fn test_12() {
        assert_eq!(
            parse_column_type(&mut Parser::new("TIMESTAMP(0)")),
            ColumnType::Timestamp(TimeAttr {
                fractional: Some(0),
            })
        );
    }

    #[test]
    fn test_13() {
        assert_eq!(
            parse_column_type(&mut Parser::new("varchar(20)")),
            ColumnType::Varchar(StringAttr {
                length: Some(20),
                charset: None,
                collation: None,
            })
        );
    }

    #[test]
    fn test_14() {
        assert_eq!(
            parse_column_type(&mut Parser::new("TEXT")),
            ColumnType::Text(StringAttr {
                length: None,
                charset: None,
                collation: None,
            })
        );
    }

    #[test]
    fn test_15() {
        assert_eq!(
            parse_column_type(&mut Parser::new(
                "TEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin"
            )),
            ColumnType::Text(StringAttr {
                length: None,
                charset: Some(CharSet::Utf8Mb4),
                collation: Some(Collation::Utf8Mb4Bin),
            })
        );
    }

    #[test]
    fn test_16() {
        assert_eq!(
            parse_column_type(&mut Parser::new("TEXT CHARACTER SET latin1")),
            ColumnType::Text(StringAttr {
                length: None,
                charset: Some(CharSet::Latin1),
                collation: None,
            })
        );
    }

    #[test]
    fn test_17() {
        assert_eq!(
            parse_column_type(&mut Parser::new("BLOB")),
            ColumnType::Blob(BlobAttr { length: None })
        );
    }

    #[test]
    fn test_18() {
        assert_eq!(
            parse_column_type(&mut Parser::new("BLOB(256)")),
            ColumnType::Blob(BlobAttr { length: Some(256) })
        );
    }

    #[test]
    fn test_19() {
        assert_eq!(
            parse_column_type(&mut Parser::new("enum('G','PG','PG-13','R','NC-17')")),
            ColumnType::Enum(EnumDef {
                values: vec![
                    "G".to_owned(),
                    "PG".to_owned(),
                    "PG-13".to_owned(),
                    "R".to_owned(),
                    "NC-17".to_owned(),
                ],
                attr: StringAttr {
                    length: None,
                    charset: None,
                    collation: None,
                }
            })
        );
    }

    #[test]
    fn test_20() {
        assert_eq!(
            parse_column_type(&mut Parser::new(
                "set('Trailers','Commentaries','Deleted Scenes','Behind the Scenes')"
            )),
            ColumnType::Set(SetDef {
                members: vec![
                    "Trailers".to_owned(),
                    "Commentaries".to_owned(),
                    "Deleted Scenes".to_owned(),
                    "Behind the Scenes".to_owned(),
                ],
                attr: StringAttr {
                    length: None,
                    charset: None,
                    collation: None,
                }
            })
        );
    }

    #[test]
    fn test_21() {
        assert_eq!(
            parse_column_type(&mut Parser::new("GEOMETRY")),
            ColumnType::Geometry(GeometryAttr { srid: None })
        );
    }

    #[test]
    fn test_22() {
        assert_eq!(
            parse_column_type(&mut Parser::new("GEOMETRY SRID 4326")),
            ColumnType::Geometry(GeometryAttr { srid: Some(4326) })
        );
    }

    #[test]
    fn test_23() {
        assert_eq!(parse_column_key("pri"), ColumnKey::Primary);
        assert_eq!(parse_column_key("uni"), ColumnKey::Unique);
        assert_eq!(parse_column_key("mul"), ColumnKey::Multiple);
        assert_eq!(parse_column_key(""), ColumnKey::NotKey);
    }

    #[test]
    fn test_24() {
        assert!(parse_column_null("yes"));
        assert!(!parse_column_null("no"));
    }
}
