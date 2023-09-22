use std::str::FromStr;

pub mod config;
pub mod generic;
pub mod promiscuous;

pub mod passive;
pub mod errors;
pub mod strategy;
pub mod aggregating;
pub mod auto_redeeming;

pub enum Strategies {
    Passive,
    Generic,
    Promiscuous,
}

impl FromStr for Strategies {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "passive" => Ok(Strategies::Passive),
            "generic" => Ok(Strategies::Generic),
            "promiscuous" => Ok(Strategies::Promiscuous),
            _ => Err(format!("No such strategy exists: {}", s)),
        }
    }
}
