use crate::mysql::def::{
    BlobAttr, EnumDef, GeometryAttr, NumericAttr, SetDef, StringAttr, TimeAttr, Type,
};
use sea_query::{EscapeBuilder, Iden, MysqlQueryBuilder};

impl Iden for Type {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        match self {
            Self::Serial => {
                write!(s, "BIGINT UNSIGNED NOT NULL AUTO_INCREMENT UNIQUE").unwrap();
            }
            Self::Bit(attr) => {
                write!(s, "BIT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::TinyInt(attr) => {
                write!(s, "TINYINT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::Bool => {
                write!(s, "BOOL").unwrap();
            }
            Self::SmallInt(attr) => {
                write!(s, "SMALLINT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::MediumInt(attr) => {
                write!(s, "MEDIUMINT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::Int(attr) => {
                write!(s, "INT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::BigInt(attr) => {
                write!(s, "BIGINT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::Decimal(attr) => {
                write!(s, "DECIMAL").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::Float(attr) => {
                write!(s, "FLOAT").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::Double(attr) => {
                write!(s, "DOUBLE").unwrap();
                Self::write_numeric_attr(s, attr);
            }
            Self::Date => {
                write!(s, "DATE").unwrap();
            }
            Self::Time(attr) => {
                write!(s, "TIME").unwrap();
                Self::write_time_attr(s, attr);
            }
            Self::DateTime(attr) => {
                write!(s, "DATETIME").unwrap();
                Self::write_time_attr(s, attr);
            }
            Self::Timestamp(attr) => {
                write!(s, "TIMESTAMP").unwrap();
                Self::write_time_attr(s, attr);
            }
            Self::Year => {
                write!(s, "YEAR").unwrap();
            }
            Self::Char(attr) => {
                write!(s, "CHAR").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::NChar(attr) => {
                write!(s, "NCHAR").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::Varchar(attr) => {
                write!(s, "VARCHAR").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::NVarchar(attr) => {
                write!(s, "NVARCHAR").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::Binary(attr) => {
                write!(s, "BINARY").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::Varbinary(attr) => {
                write!(s, "VARBINARY").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::Text(attr) => {
                write!(s, "TEXT").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::TinyText(attr) => {
                write!(s, "TINYTEXT").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::MediumText(attr) => {
                write!(s, "MEDIUMTEXT").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::LongText(attr) => {
                write!(s, "LONGTEXT").unwrap();
                Self::write_string_attr(s, attr);
            }
            Self::Blob(attr) => {
                write!(s, "BLOB").unwrap();
                Self::write_blob_attr(s, attr);
            }
            Self::TinyBlob => {
                write!(s, "TINYBLOB").unwrap();
            }
            Self::MediumBlob => {
                write!(s, "MEDIUMBLOB").unwrap();
            }
            Self::LongBlob => {
                write!(s, "LONGBLOB").unwrap();
            }
            Self::Enum(def) => {
                write!(s, "ENUM").unwrap();
                Self::write_enum_def(s, def);
            }
            Self::Set(def) => {
                write!(s, "SET").unwrap();
                Self::write_set_def(s, def);
            }
            Self::Geometry(attr) => {
                write!(s, "GEOMETRY").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::Point(attr) => {
                write!(s, "POINT").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::LineString(attr) => {
                write!(s, "LINESTRING").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::Polygon(attr) => {
                write!(s, "POLYGON").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::MultiPoint(attr) => {
                write!(s, "MULTIPOINT").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::MultiLineString(attr) => {
                write!(s, "MULTILINESTRING").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::MultiPolygon(attr) => {
                write!(s, "MULTIPOLYGON").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::GeometryCollection(attr) => {
                write!(s, "GEOMETRYCOLLECTION").unwrap();
                Self::write_geometry_attr(s, attr);
            }
            Self::Json => {
                write!(s, "JSON").unwrap();
            }
            Self::Unknown(string) => {
                write!(s, "{}", string.as_str()).unwrap();
            }
        }
    }
}

impl Type {
    pub fn write_numeric_attr(s: &mut dyn std::fmt::Write, num: &NumericAttr) {
        if num.maximum.is_some() || num.decimal.is_some() {
            write!(s, "(").unwrap();
        }
        if num.maximum.is_some() {
            write!(s, "{}", num.maximum.unwrap()).unwrap();
        }
        if num.maximum.is_some() && num.decimal.is_some() {
            write!(s, ", ").unwrap();
        }
        if num.decimal.is_some() {
            write!(s, "{}", num.decimal.unwrap()).unwrap();
        }
        if num.maximum.is_some() || num.decimal.is_some() {
            write!(s, ")").unwrap();
        }
        if num.unsigned.is_some() && num.unsigned.unwrap() {
            write!(s, " UNSIGNED").unwrap();
        }
        if num.zero_fill.is_some() && num.zero_fill.unwrap() {
            write!(s, " ZEROFILL").unwrap();
        }
    }

    pub fn write_time_attr(s: &mut dyn std::fmt::Write, attr: &TimeAttr) {
        if attr.fractional.is_some() {
            write!(s, "({})", attr.fractional.unwrap()).unwrap();
        }
    }

    pub fn write_string_attr(s: &mut dyn std::fmt::Write, attr: &StringAttr) {
        if attr.length.is_some() {
            write!(s, "({})", attr.length.unwrap()).unwrap();
        }
        if attr.charset.is_some() {
            write!(s, " CHARACTER SET ").unwrap();
            attr.charset.as_ref().unwrap().unquoted(s);
        }
        if attr.collation.is_some() {
            write!(s, " COLLATE ").unwrap();
            attr.collation.as_ref().unwrap().unquoted(s);
        }
    }

    pub fn write_blob_attr(s: &mut dyn std::fmt::Write, attr: &BlobAttr) {
        if attr.length.is_some() {
            write!(s, "({})", attr.length.unwrap()).unwrap();
        }
    }

    pub fn write_enum_def(s: &mut dyn std::fmt::Write, def: &EnumDef) {
        write!(s, " (").unwrap();
        for (i, val) in def.values.iter().enumerate() {
            if i > 0 {
                write!(s, ", ").unwrap();
            }
            write!(s, "\'{}\'", MysqlQueryBuilder.escape_string(val.as_str())).unwrap();
        }
        write!(s, ")").unwrap();
        Self::write_string_attr(s, &def.attr);
    }

    pub fn write_set_def(s: &mut dyn std::fmt::Write, def: &SetDef) {
        write!(s, " (").unwrap();
        for (i, val) in def.members.iter().enumerate() {
            if i > 0 {
                write!(s, ", ").unwrap();
            }
            write!(s, "\'{}\'", MysqlQueryBuilder.escape_string(val.as_str())).unwrap();
        }
        write!(s, ")").unwrap();
        Self::write_string_attr(s, &def.attr);
    }

    pub fn write_geometry_attr(s: &mut dyn std::fmt::Write, attr: &GeometryAttr) {
        if attr.srid.is_some() {
            write!(s, " SRID {}", attr.srid.unwrap()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mysql::def::{CharSet, Collation};

    #[test]
    fn test_1() {
        assert_eq!(
            Type::Serial.to_string().as_str(),
            "BIGINT UNSIGNED NOT NULL AUTO_INCREMENT UNIQUE"
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(Type::Bit(NumericAttr::m(1)).to_string().as_str(), "BIT(1)");
    }

    #[test]
    fn test_3() {
        assert_eq!(
            Type::TinyInt(NumericAttr::default().unsigned().take())
                .to_string()
                .as_str(),
            "TINYINT UNSIGNED"
        );
    }

    #[test]
    fn test_4() {
        assert_eq!(
            Type::TinyInt(NumericAttr::default().unsigned().zero_fill().take())
                .to_string()
                .as_str(),
            "TINYINT UNSIGNED ZEROFILL"
        );
    }

    #[test]
    fn test_5() {
        assert_eq!(Type::Bool.to_string().as_str(), "BOOL");
    }

    #[test]
    fn test_6() {
        assert_eq!(
            Type::SmallInt(NumericAttr::m(8)).to_string().as_str(),
            "SMALLINT(8)"
        );
    }

    #[test]
    fn test_7() {
        assert_eq!(
            Type::Int(NumericAttr::m(11)).to_string().as_str(),
            "INT(11)"
        );
    }

    #[test]
    fn test_8() {
        assert_eq!(
            Type::Int(NumericAttr::m(11).unsigned().take())
                .to_string()
                .as_str(),
            "INT(11) UNSIGNED"
        );
    }

    #[test]
    fn test_9() {
        assert_eq!(
            Type::BigInt(NumericAttr::m(22)).to_string().as_str(),
            "BIGINT(22)"
        );
    }

    #[test]
    fn test_10() {
        assert_eq!(
            Type::Decimal(NumericAttr::m_d(12, 8)).to_string().as_str(),
            "DECIMAL(12, 8)"
        );
    }

    #[test]
    fn test_11() {
        assert_eq!(
            Type::Decimal(NumericAttr::m(4)).to_string().as_str(),
            "DECIMAL(4)"
        );
    }

    #[test]
    fn test_12() {
        assert_eq!(
            Type::Float(NumericAttr::default()).to_string().as_str(),
            "FLOAT"
        );
        assert_eq!(
            Type::Double(NumericAttr::default()).to_string().as_str(),
            "DOUBLE"
        );
    }

    #[test]
    fn test_13() {
        assert_eq!(Type::Time(TimeAttr::default()).to_string().as_str(), "TIME");
        assert_eq!(Type::Time(TimeAttr::fsp(6)).to_string().as_str(), "TIME(6)");
    }

    #[test]
    fn test_14() {
        assert_eq!(
            Type::DateTime(TimeAttr::default()).to_string().as_str(),
            "DATETIME"
        );
        assert_eq!(
            Type::Timestamp(TimeAttr::default()).to_string().as_str(),
            "TIMESTAMP"
        );
        assert_eq!(Type::Year.to_string().as_str(), "YEAR");
    }

    #[test]
    fn test_15() {
        assert_eq!(
            Type::Char(StringAttr::default()).to_string().as_str(),
            "CHAR"
        );
        assert_eq!(
            Type::NChar(StringAttr::default()).to_string().as_str(),
            "NCHAR"
        );
        assert_eq!(
            Type::Varchar(StringAttr::default()).to_string().as_str(),
            "VARCHAR"
        );
        assert_eq!(
            Type::NVarchar(StringAttr::default()).to_string().as_str(),
            "NVARCHAR"
        );
        assert_eq!(
            Type::Binary(StringAttr::default()).to_string().as_str(),
            "BINARY"
        );
        assert_eq!(
            Type::Varbinary(StringAttr::default()).to_string().as_str(),
            "VARBINARY"
        );
        assert_eq!(
            Type::Text(StringAttr::default()).to_string().as_str(),
            "TEXT"
        );
        assert_eq!(
            Type::TinyText(StringAttr::default()).to_string().as_str(),
            "TINYTEXT"
        );
        assert_eq!(
            Type::MediumText(StringAttr::default()).to_string().as_str(),
            "MEDIUMTEXT"
        );
        assert_eq!(
            Type::LongText(StringAttr::default()).to_string().as_str(),
            "LONGTEXT"
        );
    }

    #[test]
    fn test_16() {
        assert_eq!(
            Type::Varchar(StringAttr::length(255)).to_string().as_str(),
            "VARCHAR(255)"
        );
        assert_eq!(
            Type::Varchar(StringAttr {
                length: Some(255),
                charset: Some(CharSet::Utf8Mb4),
                collation: None,
            })
            .to_string()
            .as_str(),
            "VARCHAR(255) CHARACTER SET utf8mb4"
        );
        assert_eq!(
            Type::Varchar(StringAttr {
                length: Some(255),
                charset: Some(CharSet::Utf8Mb4),
                collation: Some(Collation::Utf8Mb4Bin),
            })
            .to_string()
            .as_str(),
            "VARCHAR(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin"
        );
    }

    #[test]
    fn test_17() {
        assert_eq!(Type::Blob(BlobAttr::default()).to_string().as_str(), "BLOB");
        assert_eq!(
            Type::Blob(BlobAttr::length(128)).to_string().as_str(),
            "BLOB(128)"
        );
        assert_eq!(Type::TinyBlob.to_string().as_str(), "TINYBLOB");
        assert_eq!(Type::MediumBlob.to_string().as_str(), "MEDIUMBLOB");
        assert_eq!(Type::LongBlob.to_string().as_str(), "LONGBLOB");
    }

    #[test]
    fn test_18() {
        assert_eq!(
            Type::Enum(EnumDef {
                values: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                attr: StringAttr::default(),
            })
            .to_string()
            .as_str(),
            "ENUM ('A', 'B', 'C')"
        );

        assert_eq!(
            Type::Enum(EnumDef {
                values: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                attr: StringAttr {
                    length: None,
                    charset: Some(CharSet::Utf8Mb4),
                    collation: None,
                },
            })
            .to_string()
            .as_str(),
            "ENUM ('A', 'B', 'C') CHARACTER SET utf8mb4"
        );
    }

    #[test]
    fn test_19() {
        assert_eq!(
            Type::Geometry(GeometryAttr::srid(4326))
                .to_string()
                .as_str(),
            "GEOMETRY SRID 4326"
        );
    }

    #[test]
    fn test_20() {
        assert_eq!(
            Type::Geometry(GeometryAttr::default()).to_string().as_str(),
            "GEOMETRY"
        );
        assert_eq!(
            Type::Point(GeometryAttr::default()).to_string().as_str(),
            "POINT"
        );
        assert_eq!(
            Type::LineString(GeometryAttr::default())
                .to_string()
                .as_str(),
            "LINESTRING"
        );
        assert_eq!(
            Type::Polygon(GeometryAttr::default()).to_string().as_str(),
            "POLYGON"
        );
        assert_eq!(
            Type::MultiPoint(GeometryAttr::default())
                .to_string()
                .as_str(),
            "MULTIPOINT"
        );
        assert_eq!(
            Type::MultiLineString(GeometryAttr::default())
                .to_string()
                .as_str(),
            "MULTILINESTRING"
        );
        assert_eq!(
            Type::MultiPolygon(GeometryAttr::default())
                .to_string()
                .as_str(),
            "MULTIPOLYGON"
        );
        assert_eq!(
            Type::GeometryCollection(GeometryAttr::default())
                .to_string()
                .as_str(),
            "GEOMETRYCOLLECTION"
        );
    }

    #[test]
    fn test_21() {
        assert_eq!(Type::Json.to_string().as_str(), "JSON");
    }

    #[test]
    fn test_22() {
        assert_eq!(
            Type::Unknown("hello".to_owned()).to_string().as_str(),
            "hello"
        );
        assert_eq!(
            Type::Unknown("world(2)".to_owned()).to_string().as_str(),
            "world(2)"
        );
    }
}
