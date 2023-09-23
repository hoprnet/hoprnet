use crate::strategy::SingularStrategy;
use std::fmt::{Display, Formatter};
use crate::Strategies;

/// This strategy does nothing.
pub struct PassiveStrategy;

impl PassiveStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for PassiveStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, Strategies::Passive)
    }
}

impl SingularStrategy for PassiveStrategy {}
