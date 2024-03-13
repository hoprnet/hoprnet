use sea_query::ColumnDef;
use std::num::ParseIntError;

/// A list of the offical SQLite types as outline at the official [SQLite Docs](https://www.sqlite.org/datatype3.html)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Int,
    Integer,
    TinyInt,
    SmallInt,
    MediumInt,
    BigInt,
    UnsignedBigInt,
    Int2,
    Int8,
    Character { length: u8 },
    VarChar { length: u8 },
    VaryingCharacter { length: u8 },
    Nchar { length: u8 },
    NativeCharacter { length: u8 },
    NvarChar { length: u8 },
    Text,
    Clob,
    Blob, //No datatype specified
    Real,
    Double,
    DoublePrecision,
    Float,
    Numeric,
    Decimal { integral: u8, fractional: u8 },
    Boolean,
    Date,
    DateTime,
    Timestamp,
}

impl Type {
    /// Maps a string type from an `SqliteRow` into a [Type]
    pub fn to_type(data_type: &str) -> Result<Type, ParseIntError> {
        let data_type = data_type.to_uppercase();

        let split_type: Vec<&str> = data_type.split('(').collect();
        let type_result = match split_type[0] {
            "INT" => Type::Int,
            "INTEGER" => Type::Integer,
            "TINY INT" | "TINYINT" => Type::TinyInt,
            "SMALL INT" | "SMALLINT" => Type::SmallInt,
            "MEDIUM INT" | "MEDIUMINT" => Type::MediumInt,
            "BIG INT" | "BIGINT" => Type::BigInt,
            "UNSIGNED INT" | "UNSIGNEDBIGINT" => Type::UnsignedBigInt,
            "INT2" => Type::Int2,
            "INT8" => Type::Int8,
            "TEXT" => Type::Text,
            "CLOB" => Type::Clob,
            "BLOB" => Type::Blob,
            "REAL" => Type::Real,
            "DOUBLE" => Type::Double,
            "DOUBLE PRECISION" => Type::DoublePrecision,
            "FLOAT" => Type::Float,
            "NUMERIC" => Type::Numeric,
            "DECIMAL" => {
                let decimals = split_type[1].chars().collect::<Vec<_>>();

                let integral = decimals[0].to_string().parse::<u8>()?;
                let fractional = decimals[2].to_string().parse::<u8>()?;

                Type::Decimal {
                    integral,
                    fractional,
                }
            }
            "BOOLEAN" => Type::Boolean,
            "DATE" => Type::Date,
            "DATETIME" => Type::DateTime,
            "TIMESTAMP" => Type::Timestamp,
            _ => Type::variable_types(&split_type)?,
        };

        Ok(type_result)
    }

    /// Write a [Type] to a [ColumnDef]
    pub fn write_type(&self, column_def: &mut ColumnDef) {
        match self {
            Self::Int | Self::Integer | Self::MediumInt | Self::Int2 | Self::Int8 => {
                column_def.integer();
            }
            Self::TinyInt => {
                column_def.tiny_integer();
            }
            Self::SmallInt => {
                column_def.small_integer();
            }
            Self::BigInt | Self::UnsignedBigInt => {
                column_def.big_integer();
            }
            Self::Character { .. }
            | Self::VarChar { .. }
            | Self::VaryingCharacter { .. }
            | Self::Nchar { .. }
            | Self::NativeCharacter { .. }
            | Self::NvarChar { .. }
            | Self::Text
            | Self::Clob => {
                column_def.string();
            }
            Self::Blob => {
                column_def.binary();
            }
            Self::Real | Self::Double | Self::DoublePrecision | Self::Float | Self::Numeric => {
                column_def.double();
            }
            Self::Decimal {
                integral,
                fractional,
            } => {
                column_def.decimal_len((*integral) as u32, (*fractional) as u32);
            }
            Self::Boolean => {
                column_def.boolean();
            }
            Self::Date => {
                column_def.date();
            }
            Self::DateTime => {
                column_def.date_time();
            }
            Self::Timestamp => {
                column_def.timestamp();
            }
        }
    }

    #[allow(dead_code)]
    fn concat_type(&self, type_name: &str, length: &u8) -> String {
        let mut value = String::default();
        value.push_str(type_name);
        value.push('(');
        value.push_str(&length.to_string());
        value.push(')');

        value
    }

    fn variable_types(split_type: &[&str]) -> Result<Type, ParseIntError> {
        let length = if !split_type.len() == 1 {
            let maybe_size = split_type[1].replace(')', "");
            maybe_size.parse::<u8>()?
        } else {
            255_u8
        };

        let type_result = match split_type[0] {
            "VARCHAR" => Type::VarChar { length },
            "CHARACTER" => Type::Character { length },
            "VARYING CHARACTER" => Type::VaryingCharacter { length },
            "NCHAR" => Type::Nchar { length },
            "NATIVE CHARACTER" => Type::NativeCharacter { length },
            "NVARCHAR" => Type::NvarChar { length },
            _ => Type::Blob,
        };
        Ok(type_result)
    }
}

/// The default types for an SQLite `dflt_value`
#[derive(Debug, PartialEq, Clone)]
pub enum DefaultType {
    Integer(i32),
    Float(f32),
    String(String),
    Null,
    Unspecified, //FIXME For other types
    CurrentTimestamp,
}
