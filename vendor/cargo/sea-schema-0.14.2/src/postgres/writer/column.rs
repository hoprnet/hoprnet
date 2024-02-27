use crate::postgres::def::{ColumnInfo, Type};
use sea_query::{Alias, BlobSize, ColumnDef, ColumnType, DynIden, IntoIden, PgInterval, RcOrArc};
use std::{convert::TryFrom, fmt::Write};

impl ColumnInfo {
    pub fn write(&self) -> ColumnDef {
        let mut col_info = self.clone();
        let mut extras: Vec<String> = Vec::new();
        if let Some(default) = self.default.as_ref() {
            if default.0.starts_with("nextval") {
                col_info = Self::convert_to_serial(col_info);
            } else {
                let mut string = "".to_owned();
                write!(&mut string, "DEFAULT {}", default.0).unwrap();
                extras.push(string);
            }
        }
        let col_type = col_info.write_col_type();
        let mut col_def = ColumnDef::new_with_type(Alias::new(self.name.as_str()), col_type);
        if self.is_identity {
            col_info = Self::convert_to_serial(col_info);
        }
        if matches!(
            col_info.col_type,
            Type::SmallSerial | Type::Serial | Type::BigSerial
        ) {
            col_def.auto_increment();
        }
        if self.not_null.is_some() {
            col_def.not_null();
        }
        if !extras.is_empty() {
            col_def.extra(extras.join(" "));
        }
        col_def
    }

    fn convert_to_serial(mut col_info: ColumnInfo) -> ColumnInfo {
        match col_info.col_type {
            Type::SmallInt => {
                col_info.col_type = Type::SmallSerial;
            }
            Type::Integer => {
                col_info.col_type = Type::Serial;
            }
            Type::BigInt => {
                col_info.col_type = Type::BigSerial;
            }
            _ => {}
        };
        col_info
    }

    pub fn write_col_type(&self) -> ColumnType {
        fn write_type(col_type: &Type) -> ColumnType {
            match col_type {
                Type::SmallInt => ColumnType::SmallInteger,
                Type::Integer => ColumnType::Integer,
                Type::BigInt => ColumnType::BigInteger,
                Type::Decimal(num_attr) | Type::Numeric(num_attr) => {
                    match (num_attr.precision, num_attr.scale) {
                        (None, None) => ColumnType::Decimal(None),
                        (precision, scale) => ColumnType::Decimal(Some((
                            precision.unwrap_or(0).into(),
                            scale.unwrap_or(0).into(),
                        ))),
                    }
                }
                Type::Real => ColumnType::Float,
                Type::DoublePrecision => ColumnType::Double,
                Type::SmallSerial => ColumnType::SmallInteger,
                Type::Serial => ColumnType::Integer,
                Type::BigSerial => ColumnType::BigInteger,
                Type::Money => ColumnType::Money(None),
                Type::Varchar(string_attr) => {
                    ColumnType::String(string_attr.length.map(Into::into))
                }
                Type::Char(string_attr) => ColumnType::Char(string_attr.length.map(Into::into)),
                Type::Text => ColumnType::Text,
                Type::Bytea => ColumnType::Binary(BlobSize::Blob(None)),
                // The SQL standard requires that writing just timestamp be equivalent to timestamp without time zone,
                // and PostgreSQL honors that behavior. (https://www.postgresql.org/docs/current/datatype-datetime.html)
                Type::Timestamp(_) => ColumnType::DateTime,
                Type::TimestampWithTimeZone(_) => ColumnType::TimestampWithTimeZone,
                Type::Date => ColumnType::Date,
                Type::Time(_) => ColumnType::Time,
                Type::TimeWithTimeZone(_) => ColumnType::Time,
                Type::Interval(interval_attr) => {
                    let field = match &interval_attr.field {
                        Some(field) => PgInterval::try_from(field).ok(),
                        None => None,
                    };
                    let precision = interval_attr.precision.map(Into::into);
                    ColumnType::Interval(field, precision)
                }
                Type::Boolean => ColumnType::Boolean,
                Type::Point => ColumnType::Custom(Alias::new("point").into_iden()),
                Type::Line => ColumnType::Custom(Alias::new("line").into_iden()),
                Type::Lseg => ColumnType::Custom(Alias::new("lseg").into_iden()),
                Type::Box => ColumnType::Custom(Alias::new("box").into_iden()),
                Type::Path => ColumnType::Custom(Alias::new("path").into_iden()),
                Type::Polygon => ColumnType::Custom(Alias::new("polygon").into_iden()),
                Type::Circle => ColumnType::Custom(Alias::new("circle").into_iden()),
                Type::Cidr => ColumnType::Custom(Alias::new("cidr").into_iden()),
                Type::Inet => ColumnType::Custom(Alias::new("inet").into_iden()),
                Type::MacAddr => ColumnType::Custom(Alias::new("macaddr").into_iden()),
                Type::MacAddr8 => ColumnType::Custom(Alias::new("macaddr8").into_iden()),
                Type::Bit(bit_attr) => {
                    let mut str = String::new();
                    write!(str, "bit").unwrap();
                    if bit_attr.length.is_some() {
                        write!(str, "(").unwrap();
                        if let Some(length) = bit_attr.length {
                            write!(str, "{}", length).unwrap();
                        }
                        write!(str, ")").unwrap();
                    }
                    ColumnType::Custom(Alias::new(&str).into_iden())
                }
                Type::TsVector => ColumnType::Custom(Alias::new("tsvector").into_iden()),
                Type::TsQuery => ColumnType::Custom(Alias::new("tsquery").into_iden()),
                Type::Uuid => ColumnType::Uuid,
                Type::Xml => ColumnType::Custom(Alias::new("xml").into_iden()),
                Type::Json => ColumnType::Json,
                Type::JsonBinary => ColumnType::JsonBinary,
                Type::Int4Range => ColumnType::Custom(Alias::new("int4range").into_iden()),
                Type::Int8Range => ColumnType::Custom(Alias::new("int8range").into_iden()),
                Type::NumRange => ColumnType::Custom(Alias::new("numrange").into_iden()),
                Type::TsRange => ColumnType::Custom(Alias::new("tsrange").into_iden()),
                Type::TsTzRange => ColumnType::Custom(Alias::new("tstzrange").into_iden()),
                Type::DateRange => ColumnType::Custom(Alias::new("daterange").into_iden()),
                Type::PgLsn => ColumnType::Custom(Alias::new("pg_lsn").into_iden()),
                Type::Unknown(s) => ColumnType::Custom(Alias::new(s).into_iden()),
                Type::Enum(enum_def) => {
                    let name = Alias::new(&enum_def.typename).into_iden();
                    let variants: Vec<DynIden> = enum_def
                        .values
                        .iter()
                        .map(|variant| Alias::new(variant).into_iden())
                        .collect();
                    ColumnType::Enum { name, variants }
                }
                Type::Array(array_def) => ColumnType::Array(RcOrArc::new(write_type(
                    array_def.col_type.as_ref().expect("Array type not defined"),
                ))),
            }
        }
        write_type(&self.col_type)
    }
}
