#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use crate as sea_schema;

#[derive(Clone, Debug, PartialEq, sea_query::Iden, sea_schema_derive::Name)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[catch = "string_to_unknown"]
pub enum StorageEngine {
    #[iden = "ARCHIVE"]
    Archive,
    #[iden = "BLACKHOLE"]
    Blackhole,
    #[iden = "MRG_MYISAM"]
    MrgMyIsam,
    #[iden = "FEDERATED"]
    Federated,
    #[iden = "MyISAM"]
    MyIsam,
    #[iden = "PERFORMANCE_SCHEMA"]
    PerformanceSchema,
    #[iden = "InnoDB"]
    InnoDb,
    #[iden = "MEMORY"]
    Memory,
    #[iden = "CSV"]
    Csv,
    #[method = "unknown_to_string"]
    Unknown(String),
}

impl StorageEngine {
    pub fn unknown_to_string(&self) -> &String {
        match self {
            Self::Unknown(custom) => custom,
            _ => panic!("not Unknown"),
        }
    }

    pub fn string_to_unknown(string: &str) -> Option<Self> {
        Some(Self::Unknown(string.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_0() {
        assert_eq!(
            StorageEngine::from_str("ARCHIVE").unwrap(),
            StorageEngine::Archive
        );
        assert_eq!(
            StorageEngine::from_str("InnoDB").unwrap(),
            StorageEngine::InnoDb
        );
        assert_eq!(
            StorageEngine::from_str("MyISAM").unwrap(),
            StorageEngine::MyIsam
        );
    }

    #[test]
    fn test_1() {
        assert_eq!(
            StorageEngine::from_str("hello").unwrap(),
            StorageEngine::Unknown("hello".to_owned())
        );
    }
}
