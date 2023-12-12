#[cfg(feature = "serdejson")]
use base64::{decode, encode, DecodeError};
#[cfg(feature = "serdevalid")]
use paste;
#[cfg(feature = "serdevalid")]
use regex::Regex;
#[cfg(feature = "serdejson")]
use serde::de::{Deserialize, Deserializer, Error};
#[cfg(feature = "serdejson")]
use serde::ser::{Serialize, Serializer};
#[cfg(feature = "serdevalid")]
use serde_valid;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// Base64-encoded byte array
pub struct ByteArray(pub Vec<u8>);

#[cfg(feature = "serdejson")]
impl Serialize for ByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode(&self.0))
    }
}

#[cfg(feature = "serdejson")]
impl<'de> Deserialize<'de> for ByteArray {
    fn deserialize<D>(deserializer: D) -> Result<ByteArray, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match decode(s) {
            Ok(bin) => Ok(ByteArray(bin)),
            _ => Err(D::Error::custom("invalid base64")),
        }
    }
}

// Validation macro to create impls for serde_valid integration.
#[cfg(feature = "serdevalid")]
macro_rules! impl_validate_byte_array {
    (
        $ErrorType:ident,
        $DiveFn:expr,
        $limit_type:ty$(,)*

    ) => {
        paste::paste! {
            #[cfg(feature = "serdevalid")]
            impl serde_valid::[<Validate $ErrorType >] for ByteArray {
                fn [<validate_ $ErrorType:snake>](&self, limit: $limit_type) -> Result<(), serde_valid::[<$ErrorType Error>]> {
                    self.$DiveFn.[<validate_ $ErrorType:snake>](limit)
                }
            }
        }
    };
}

// Allow validation of encoded string.
#[cfg(feature = "serdevalid")]
impl_validate_byte_array!(Pattern, to_string(), &Regex);
#[cfg(feature = "serdevalid")]
impl_validate_byte_array!(MaxLength, to_string(), usize);
#[cfg(feature = "serdevalid")]
impl_validate_byte_array!(MinLength, to_string(), usize);

#[cfg(feature = "serdevalid")]
impl serde_valid::ValidateEnumerate<&'static str> for ByteArray {
    fn validate_enumerate(
        &self,
        enumerate: &[&'static str],
    ) -> Result<(), serde_valid::EnumerateError> {
        self.to_string().validate_enumerate(enumerate)
    }
}

// Also allow validation decoded internals.
#[cfg(feature = "serdevalid")]
impl_validate_byte_array!(MaxItems, 0, usize);
#[cfg(feature = "serdevalid")]
impl_validate_byte_array!(MinItems, 0, usize);

#[cfg(feature = "serdevalid")]
impl serde_valid::ValidateUniqueItems for ByteArray {
    fn validate_unique_items(&self) -> Result<(), serde_valid::UniqueItemsError> {
        self.0.validate_unique_items()
    }
}

impl std::str::FromStr for ByteArray {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(decode(s)?))
    }
}

impl ToString for ByteArray {
    fn to_string(&self) -> String {
        encode(&self.0)
    }
}

impl Deref for ByteArray {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for ByteArray {
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

#[cfg(test)]
#[cfg(feature = "serdevalid")]
mod serde_tests {
    use super::*;
    use serde_json::json;
    use serde_valid::Validate;
    use std::str::FromStr;

    #[derive(Validate)]
    struct ValidateByteArrayStruct {
        // Validate encoded as string
        #[validate(enumerate("YWJjZGU="))]
        #[validate(max_length = 8)]
        #[validate(min_length = 8)]
        #[validate(pattern = ".*=")]
        // Validate decoded as Vec (b"abcde")
        #[validate(max_items = 5)]
        #[validate(min_items = 5)]
        #[validate(unique_items)]
        byte_array: ByteArray,
    }

    #[test]
    fn valid_byte_array() {
        let test_struct = ValidateByteArrayStruct {
            byte_array: ByteArray::from_str("YWJjZGU=").unwrap(),
        };
        assert!(test_struct.validate().is_ok());
    }

    #[test]
    fn invalid_few_byte_array() {
        let test_struct = ValidateByteArrayStruct {
            byte_array: ByteArray::from_str("ZmZm").unwrap(),
        };
        let errors = test_struct.validate().unwrap_err().to_string();
        assert_eq!(
            errors,
            json!({"errors":[],
                "properties":{
                    "byte_array":{
                        "errors":[
                            "The value must be in [YWJjZGU=].",
                            "The length of the value must be `>= 8`.",
                            "The value must match the pattern of \".*=\".",
                            "The length of the items must be `>= 5`.",
                            "The items must be unique."
                            ]
                        }
                    }
                }
            )
            .to_string()
        );
    }

    #[test]
    fn invalid_many_byte_array() {
        let test_struct = ValidateByteArrayStruct {
            byte_array: ByteArray::from_str("ZmZmZmZmZg==").unwrap(),
        };
        let errors = test_struct.validate().unwrap_err().to_string();
        assert_eq!(
            errors,
            json!({"errors":[],
                "properties":{
                    "byte_array":{
                        "errors":[
                            "The value must be in [YWJjZGU=].",
                            "The length of the value must be `<= 8`.",
                            "The length of the items must be `<= 5`.",
                            "The items must be unique."
                            ]
                        }
                    }
                }
            )
            .to_string()
        );
    }
}
