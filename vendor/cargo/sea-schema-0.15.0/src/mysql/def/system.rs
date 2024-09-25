#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct SystemInfo {
    /// The version number converted to integer using the following formula:
    /// major_version * 10000 + minor_version * 100 + sub_version
    pub version: u32,
    /// The system string. it may be: `0ubuntu0.*` or `MariaDB`
    pub system: String,
    /// Additional suffix
    pub suffix: Vec<String>,
}

impl SystemInfo {
    /// Return true if the system is MariaDB
    pub fn is_maria_db(&self) -> bool {
        self.system == "MariaDB"
    }

    /// Return true if the system is not MariaDB
    pub fn is_mysql(&self) -> bool {
        !self.is_maria_db()
    }

    /// Return the version version as string. e.g. 8.0.1
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.version / 10000,
            self.version / 100 % 100,
            self.version % 100
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0() {
        let system = SystemInfo {
            version: 50110,
            ..Default::default()
        };
        assert_eq!(system.version_string(), "5.1.10".to_owned());
    }

    #[test]
    fn test_1() {
        let system = SystemInfo {
            version: 80023,
            ..Default::default()
        };
        assert_eq!(system.version_string(), "8.0.23".to_owned());
    }
}
