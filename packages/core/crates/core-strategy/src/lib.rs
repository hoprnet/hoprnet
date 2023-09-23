use strum::{Display, EnumString};

pub mod config;
pub mod decision;
pub mod promiscuous;

pub mod aggregating;
pub mod auto_funding;
pub mod auto_redeeming;
pub mod errors;
pub mod passive;
pub mod strategy;

#[derive(Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Strategies {
    Passive,
    Promiscuous,
    Aggregating,
    AutoRedeeming,
    AutoFunding,
}
