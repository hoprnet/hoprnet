use crate::mysql::def::*;
use crate::mysql::query::VersionQueryResult;

impl VersionQueryResult {
    pub fn parse(self) -> SystemInfo {
        parse_version_query_result(self)
    }
}

pub fn parse_version_query_result(result: VersionQueryResult) -> SystemInfo {
    parse_version_string(result.version.as_str())
}

pub fn parse_version_string(string: &str) -> SystemInfo {
    let mut system = SystemInfo::default();
    for (i, part) in string.split('-').enumerate() {
        if i == 0 {
            system.version = parse_version_number(part);
        } else if i == 1 {
            system.system = part.to_owned();
        } else {
            system.suffix.push(part.to_owned());
        }
    }
    system
}

pub fn parse_version_number(string: &str) -> u32 {
    let mut number: u32 = 0;
    let numbers: Vec<&str> = string.split('.').collect();
    #[allow(clippy::len_zero)]
    if numbers.len() > 0 {
        number += numbers[0].parse::<u32>().unwrap() * 10000
    }
    if numbers.len() > 1 {
        number += numbers[1].parse::<u32>().unwrap() * 100
    }
    if numbers.len() > 2 {
        number += numbers[2].parse::<u32>().unwrap()
    }
    number
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0() {
        assert_eq!(parse_version_number("5.1.10"), 50110);
    }

    #[test]
    fn test_1() {
        assert_eq!(parse_version_number("8.0.23"), 80023);
    }

    #[test]
    fn test_2() {
        assert_eq!(
            parse_version_string("8.0.23-0ubuntu0.20.04.1"),
            SystemInfo {
                version: 80023,
                system: "0ubuntu0.20.04.1".to_owned(),
                suffix: vec![],
            }
        )
    }

    #[test]
    fn test_3() {
        assert_eq!(
            parse_version_string("10.2.31-MariaDB"),
            SystemInfo {
                version: 100231,
                system: "MariaDB".to_owned(),
                suffix: vec![],
            }
        )
    }

    #[test]
    fn test_4() {
        assert_eq!(
            parse_version_string("10.2.31-MariaDB-debug"),
            SystemInfo {
                version: 100231,
                system: "MariaDB".to_owned(),
                suffix: vec!["debug".to_owned()],
            }
        )
    }
}
