use sea_query::{ColumnType, StringLen};
use std::num::ParseIntError;

pub fn parse_type(data_type: &str) -> Result<ColumnType, ParseIntError> {
    let mut type_name = data_type;
    let mut parts: Vec<u32> = Vec::new();
    if let Some((prefix, suffix)) = data_type.split_once('(') {
        if let Some(suffix) = suffix.strip_suffix(')') {
            type_name = prefix;
            for part in suffix.split(',') {
                if let Ok(part) = part.trim().parse() {
                    parts.push(part);
                } else {
                    break;
                }
            }
        }
    }
    Ok(match type_name.to_lowercase().as_str() {
        "char" => ColumnType::Char(parts.into_iter().next()),
        "varchar" => ColumnType::String(match parts.into_iter().next() {
            Some(length) => StringLen::N(length),
            None => StringLen::None,
        }),
        "text" => ColumnType::Text,
        "tinyint" => ColumnType::TinyInteger,
        "smallint" => ColumnType::SmallInteger,
        "int" | "integer" => ColumnType::Integer,
        "bigint" => ColumnType::BigInteger,
        "float" => ColumnType::Float,
        "double" => ColumnType::Double,
        "decimal" | "real" => ColumnType::Decimal(if parts.len() == 2 {
            Some((parts[0], parts[1]))
        } else {
            None
        }),
        "datetime_text" => ColumnType::DateTime,
        "timestamp" | "timestamp_text" => ColumnType::Timestamp,
        "timestamp_with_timezone_text" => ColumnType::TimestampWithTimeZone,
        "time_text" => ColumnType::Time,
        "date_text" => ColumnType::Date,
        "blob" => {
            if parts.len() == 1 {
                ColumnType::Binary(parts[0])
            } else {
                ColumnType::Blob
            }
        }
        "varbinary_blob" if parts.len() == 1 => {
            ColumnType::VarBinary(match parts.into_iter().next() {
                Some(length) => StringLen::N(length),
                None => StringLen::None,
            })
        }
        "boolean" => ColumnType::Boolean,
        "real_money" => ColumnType::Money(if parts.len() == 2 {
            Some((parts[0], parts[1]))
        } else {
            None
        }),
        "json_text" => ColumnType::Json,
        "jsonb_text" => ColumnType::JsonBinary,
        "uuid_text" => ColumnType::Uuid,
        _ => ColumnType::custom(data_type),
    })
}

/// The default types for an SQLite `dflt_value`
#[derive(Debug, PartialEq, Clone)]
pub enum DefaultType {
    Integer(i32),
    Float(f32),
    String(String),
    Null,
    Unspecified,
    CurrentTimestamp,
}
