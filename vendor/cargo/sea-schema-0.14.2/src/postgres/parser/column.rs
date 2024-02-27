use crate::postgres::{
    def::*, discovery::EnumVariantMap, parser::yes_or_no_to_bool, query::ColumnQueryResult,
};
use sea_query::RcOrArc;

impl ColumnQueryResult {
    pub fn parse(self, enums: &EnumVariantMap) -> ColumnInfo {
        parse_column_query_result(self, enums)
    }
}

pub fn parse_column_query_result(result: ColumnQueryResult, enums: &EnumVariantMap) -> ColumnInfo {
    ColumnInfo {
        name: result.column_name.clone(),
        col_type: parse_column_type(&result, enums),
        default: ColumnExpression::from_option_string(result.column_default),
        generated: ColumnExpression::from_option_string(result.column_generated),
        not_null: NotNull::from_bool(!yes_or_no_to_bool(&result.is_nullable)),
        is_identity: yes_or_no_to_bool(&result.is_identity),
    }
}

pub fn parse_column_type(result: &ColumnQueryResult, enums: &EnumVariantMap) -> ColumnType {
    let is_enum = result
        .udt_name
        .as_ref()
        .map_or(false, |udt_name| enums.contains_key(udt_name));
    let mut ctype = Type::from_str(
        result.column_type.as_str(),
        result.udt_name.as_deref(),
        is_enum,
    );

    if ctype.has_numeric_attr() {
        ctype = parse_numeric_attributes(
            result.numeric_precision,
            result.numeric_precision_radix,
            result.numeric_scale,
            ctype,
        );
    }
    if ctype.has_string_attr() {
        ctype = parse_string_attributes(result.character_maximum_length, ctype);
    }
    if ctype.has_time_attr() {
        ctype = parse_time_attributes(result.datetime_precision, ctype);
    }
    if ctype.has_interval_attr() {
        ctype = parse_interval_attributes(&result.interval_type, result.interval_precision, ctype);
    }
    if ctype.has_bit_attr() {
        ctype = parse_bit_attributes(result.character_maximum_length, ctype);
    }
    if ctype.has_enum_attr() {
        ctype = parse_enum_attributes(result.udt_name.as_deref(), ctype, enums);
    }
    if ctype.has_array_attr() {
        ctype = parse_array_attributes(result.udt_name_regtype.as_deref(), ctype, enums);
    }

    ctype
}

pub fn parse_numeric_attributes(
    num_precision: Option<i32>,
    num_precision_radix: Option<i32>,
    num_scale: Option<i32>,
    mut ctype: ColumnType,
) -> ColumnType {
    let numeric_precision: Option<u16> = match num_precision {
        None => None,
        Some(num) => match u16::try_from(num) {
            Ok(num) => Some(num),
            Err(_) => None,
        },
    };
    let _numeric_precision_radix: Option<u16> = match num_precision_radix {
        None => None,
        Some(num) => match u16::try_from(num) {
            Ok(num) => Some(num),
            Err(_) => None,
        },
    };
    let numeric_scale: Option<u16> = match num_scale {
        None => None,
        Some(num) => match u16::try_from(num) {
            Ok(num) => Some(num),
            Err(_) => None,
        },
    };

    match ctype {
        Type::Decimal(ref mut attr) | Type::Numeric(ref mut attr) => {
            attr.precision = numeric_precision;
            attr.scale = numeric_scale;
        }
        _ => panic!("parse_numeric_attributes(_) received a type other than Decimal or Numeric"),
    };

    ctype
}

pub fn parse_string_attributes(
    character_maximum_length: Option<i32>,
    mut ctype: ColumnType,
) -> ColumnType {
    match ctype {
        Type::Varchar(ref mut attr) | Type::Char(ref mut attr) => {
            attr.length = match character_maximum_length {
                None => None,
                Some(num) => match u16::try_from(num) {
                    Ok(num) => Some(num),
                    Err(_) => None,
                },
            };
        }
        _ => panic!("parse_string_attributes(_) received a type that does not have StringAttr"),
    };

    ctype
}

pub fn parse_time_attributes(datetime_precision: Option<i32>, mut ctype: ColumnType) -> ColumnType {
    match ctype {
        Type::Timestamp(ref mut attr)
        | Type::TimestampWithTimeZone(ref mut attr)
        | Type::Time(ref mut attr)
        | Type::TimeWithTimeZone(ref mut attr) => {
            attr.precision = match datetime_precision {
                None => None,
                Some(num) => match u16::try_from(num) {
                    Ok(num) => Some(num),
                    Err(_) => None,
                },
            };
        }
        _ => panic!("parse_time_attributes(_) received a type that does not have TimeAttr"),
    };

    ctype
}

pub fn parse_interval_attributes(
    interval_type: &Option<String>,
    interval_precision: Option<i32>,
    mut ctype: ColumnType,
) -> ColumnType {
    match ctype {
        Type::Interval(ref mut attr) => {
            attr.field = interval_type.clone();
            attr.precision = match interval_precision {
                None => None,
                Some(num) => match u16::try_from(num) {
                    Ok(num) => Some(num),
                    Err(_) => None,
                },
            };
        }
        _ => panic!("parse_interval_attributes(_) received a type that does not have IntervalAttr"),
    };

    ctype
}

pub fn parse_bit_attributes(
    character_maximum_length: Option<i32>,
    mut ctype: ColumnType,
) -> ColumnType {
    match ctype {
        Type::Bit(ref mut attr) => {
            attr.length = match character_maximum_length {
                None => None,
                Some(num) => match u16::try_from(num) {
                    Ok(num) => Some(num),
                    Err(_) => None,
                },
            };
        }
        _ => panic!("parse_bit_attributes(_) received a type that does not have BitAttr"),
    };

    ctype
}

pub fn parse_enum_attributes(
    udt_name: Option<&str>,
    mut ctype: ColumnType,
    enums: &EnumVariantMap,
) -> ColumnType {
    match ctype {
        Type::Enum(ref mut def) => {
            def.typename = match udt_name {
                None => panic!("parse_enum_attributes(_) received an empty udt_name"),
                Some(typename) => typename.to_string(),
            };
            if let Some(variants) = enums.get(&def.typename) {
                def.values = variants.clone()
            }
        }
        _ => panic!("parse_enum_attributes(_) received a type that does not have EnumDef"),
    };

    ctype
}

pub fn parse_array_attributes(
    udt_name_regtype: Option<&str>,
    mut ctype: ColumnType,
    enums: &EnumVariantMap,
) -> ColumnType {
    match ctype {
        Type::Array(ref mut def) => {
            def.col_type = match udt_name_regtype {
                None => panic!("parse_array_attributes(_) received an empty udt_name_regtype"),
                Some(typename) => {
                    let typename = &typename.replacen('"', "", 2).replacen("[]", "", 1);
                    let arr_col_type = if let Some(variants) = enums.get(typename) {
                        Type::Enum(EnumDef {
                            typename: typename.to_string(),
                            values: variants.clone(),
                        })
                    } else {
                        Type::from_str(typename, Some(typename), false)
                    };
                    Some(RcOrArc::new(arr_col_type))
                }
            };
        }
        _ => panic!("parse_array_attributes(_) received a type that does not have EnumDef"),
    };

    ctype
}
